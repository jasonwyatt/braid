use iron::prelude::*;
use std::i64;
use iron::status;
use iron::headers::{Headers, ContentType};
use iron::typemap::{Key, TypeMap};
use router::Router;
use nutrino::{Datastore, Error, Type, Weight};
use util::SimpleError;
use common::ProxyTransaction;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use core::str::FromStr;
use iron::modifiers::Header as HeaderModifier;
use iron::mime::{Mime, TopLevel, SubLevel, Attr, Value};
use iron::request::Body;
use std::io;
use std::io::Read;
use chrono::naive::datetime::NaiveDateTime;
use serde_json::value::Value as JsonValue;
use serde_json;
use urlencoded::UrlEncodedQuery;
use serde::ser::Serialize;
use std::collections::HashMap;
use std::cmp::min;
use std::u16;
use datastore::DATASTORE;
use uuid::Uuid;

const MAX_RETURNABLE_EDGES: u16 = 1000;

// Need this to avoid orphan rules
pub struct AccountKey {
	pub account_id: Uuid
}

impl Key for AccountKey {
	type Value = AccountKey;
}


pub fn create_iron_error(status_code: status::Status, err: String) -> IronError {
	let mut d: BTreeMap<String, String> = BTreeMap::new();
	d.insert("error".to_string(), err.clone());
	let body = serde_json::to_string(&d).unwrap();
	let json_content_type_modifier = HeaderModifier(get_json_content_type());
	let modifiers = (status_code, json_content_type_modifier, body);
	IronError::new(SimpleError::new(err), modifiers)
}

pub fn get_json_content_type() -> ContentType {
	ContentType(Mime(TopLevel::Application, SubLevel::Json, vec![(Attr::Charset, Value::Utf8)]))
}

pub fn to_response<T: Serialize>(status_code: status::Status, body: &T) -> Response {
	let mut hs = Headers::new();
	hs.set(get_json_content_type());

	Response {
		status: Some(status_code),
		headers: hs,
		extensions: TypeMap::new(),
		body: Some(Box::new(serde_json::to_string(&body).unwrap()))
	}
}

pub fn get_url_param<T: FromStr>(req: &Request, name: &str) -> Result<T, IronError> {
	let s = req.extensions.get::<Router>().unwrap().find(name).unwrap();

	match T::from_str(s) {
		Ok(val) => Ok(val),
		Err(_) => Err(create_iron_error(status::BadRequest, format!("Invalid value for URL param {}", name)))
	}
}

pub fn get_required_json_string_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<String, IronError> {
	match json.get(name) {
		Some(&JsonValue::String(ref val)) => Ok(val.clone()),
		None | Some(&JsonValue::Null) => {
			Err(create_iron_error(status::BadRequest, format!("Missing `{}`", name)))
		},
		_ => {
			Err(create_iron_error(status::BadRequest, format!("Invalid type for `{}`", name)))
		}
	}
}

pub fn get_required_json_f64_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<f64, IronError> {
	match json.get(name) {
		Some(&JsonValue::F64(ref val)) => Ok(val.clone()),
		None | Some(&JsonValue::Null) => {
			Err(create_iron_error(status::BadRequest, format!("Missing `{}`", name)))
		},
		_ => {
			Err(create_iron_error(status::BadRequest, format!("Invalid type for `{}`", name)))
		}
	}
}

pub fn json_u64_to_i64(name: &str, val: u64) -> Result<i64, IronError> {
	if val > i64::MAX as u64 {
		Err(create_iron_error(status::BadRequest, format!("Invalid type for `{}`", name)))
	} else {
		Ok(val as i64)
	}
}

pub fn get_required_json_uuid_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<Uuid, IronError> {
	let s = try!(get_required_json_string_param(json, name));

	match Uuid::from_str(&s[..]) {
		Ok(u) => Ok(u),
		Err(_) => Err(create_iron_error(status::BadRequest, format!("Invalid uuid format for `{}`", name)))
	}
}

pub fn get_required_json_type_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<Type, IronError> {
	let s = try!(get_required_json_string_param(json, name));

	match Type::from_str(&s[..]) {
		Ok(u) => Ok(u),
		Err(_) => Err(create_iron_error(status::BadRequest, format!("Invalid type format for `{}`", name)))
	}
}

pub fn get_required_json_weight_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<Weight, IronError> {
	let w = try!(get_required_json_f64_param(json, name));

	match Weight::new(w as f32) {
		Ok(w) => Ok(w),
		Err(_) => Err(create_iron_error(status::BadRequest, format!("Invalid weight format for `{}`: it should be a float between -1.0 and 1.0 inclusive.", name)))
	}
}

pub fn get_optional_json_i64_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<Option<i64>, IronError> {
	match json.get(name) {
		Some(&JsonValue::I64(ref val)) => Ok(Some(val.clone())),
		Some(&JsonValue::U64(ref val)) => Ok(Some(try!(json_u64_to_i64(name, val.clone())))),
		None | Some(&JsonValue::Null) => Ok(None),
		_ => {
			Err(create_iron_error(status::BadRequest, format!("Invalid type for `{}`", name)))
		}
	}
}

pub fn get_optional_json_u64_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<Option<u64>, IronError> {
	match json.get(name) {
		Some(&JsonValue::I64(ref val)) => {
			if *val < 0 {
				Err(create_iron_error(status::BadRequest, format!("Invalid type for `{}`", name)))
			} else {
				Ok(Some(val.clone() as u64))
			}
		}
		Some(&JsonValue::U64(ref val)) => Ok(Some(val.clone())),
		None | Some(&JsonValue::Null) => Ok(None),
		_ => {
			Err(create_iron_error(status::BadRequest, format!("Invalid type for `{}`", name)))
		}
	}
}

pub fn get_optional_json_u16_param(json: &BTreeMap<String, JsonValue>, name: &str) -> Result<Option<u16>, IronError> {
	match try!(get_optional_json_u64_param(json, name)) {
		Some(val) if val > u16::MAX as u64 => Err(create_iron_error(status::BadRequest, format!("Invalid type for `{}`", name))),
		Some(val) => Ok(Some(val as u16)),
		None => Ok(None)
	}
}

pub fn parse_limit(val: Option<u16>) -> u16 {
	match val {
		Some(val) => min(val, MAX_RETURNABLE_EDGES),
		_ => MAX_RETURNABLE_EDGES
	}
}

pub fn parse_datetime(val: Option<i64>) -> Option<NaiveDateTime> {
	match val {
		Some(val) => Some(NaiveDateTime::from_timestamp(val, 0)),
		_ => None
	}
}

pub fn datastore_request<T>(result: Result<T, Error>) -> Result<T, IronError> {
	match result {
		Ok(result) => Ok(result),
		Err(err) => {
			let status = match err {
				Error::AccountNotFound | Error::VertexNotFound | Error::EdgeNotFound | Error::MetadataNotFound => status::NotFound,
				Error::OutOfRange(_) => status::BadRequest,
				Error::Unexpected(_) => status::InternalServerError
			};

			Err(create_iron_error(status, format!("{}", err)))
		}
	}
}

pub fn get_account_id(req: &Request) -> Uuid {
	let ext = &(*req.extensions.get::<AccountKey>().unwrap());
	ext.account_id
}

pub fn get_transaction(req: &Request) -> Result<ProxyTransaction, IronError> {
	let account_id = get_account_id(req);
	match DATASTORE.transaction(account_id) {
		Ok(val) => Ok(val),
		Err(err) => Err(create_iron_error(status::InternalServerError, format!("Could not create datastore transaction: {}", err)))
	}
}

pub fn read_optional_json(body: &mut Body) -> Result<Option<JsonValue>, IronError> {
	let mut payload = String::new();
	let read_result: Result<usize, io::Error> = body.read_to_string(&mut payload);

	if let Err(err) = read_result {
		return Err(create_iron_error(status::BadRequest, format!("Could not read JSON body: {}", err)))
	}

	if payload.len() == 0 {
		Ok(None)
	} else {
		match serde_json::from_str(&payload[..]) {
			Ok(json) => Ok(Some(json)),
			Err(err) => Err(create_iron_error(status::BadRequest, format!("Could not parse JSON payload: {}", err.description())))
		}
	}	
}

pub fn read_required_json(mut body: &mut Body) -> Result<JsonValue, IronError> {
	match try!(read_optional_json(&mut body)) {
		Some(value) => Ok(value),
		None => Err(create_iron_error(status::BadRequest, "Missing JSON payload".to_string())),
	}
}

pub fn read_json_object(body: &mut Body) -> Result<BTreeMap<String, JsonValue>, IronError> {
	match try!(read_required_json(body)) {
		JsonValue::Object(obj) => Ok(obj),
		_ => Err(create_iron_error(status::BadRequest, "Bad payload: expected object".to_string()))
	}
}

pub fn get_query_params<'a>(req: &'a mut Request) -> Result<&'a HashMap<String, Vec<String>>, IronError> {
	match req.get_ref::<UrlEncodedQuery>() {
        Ok(map) => Ok(map),
        Err(_) => Err(create_iron_error(status::BadRequest, "Could not parse query parameters".to_string()))
    }
}

pub fn get_query_param<T: FromStr>(params: &HashMap<String, Vec<String>>, key: String, required: bool) -> Result<Option<T>, IronError> {
	if let Some(values) = params.get(&key) {
		if let Some(first_value) = values.get(0) {
			match first_value.parse::<T>() {
				Ok(value) => return Ok(Some(value)),
				Err(_) => return Err(create_iron_error(status::BadRequest, format!("Could not parse query parameter `{}`", key)))
			}
		}
	}

	if required {
		Err(create_iron_error(status::BadRequest, format!("Missing required query parameter `{}`", key)))
	} else {
		Ok(None)
	}
}