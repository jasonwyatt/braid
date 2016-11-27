use lua;
use serde_json::value::Value as JsonValue;
use std::collections::BTreeMap;
use nutrino::{Edge, Type, Weight};
use chrono::naive::datetime::NaiveDateTime;
use std::{isize, i32, u16};
use uuid::Uuid;
use core::str::FromStr;
use super::errors::LuaError;

pub unsafe fn deserialize_json(l: &mut lua::ExternState, offset: i32) -> Result<JsonValue, LuaError> {
    Ok(match l.type_(offset) {
        Some(lua::Type::Nil) | None => JsonValue::Null,
        Some(lua::Type::Boolean) => JsonValue::Bool(l.toboolean(-1)),
        Some(lua::Type::Number) => JsonValue::F64(l.tonumber(-1)),
        Some(lua::Type::String) => JsonValue::String(l.checkstring(-1).unwrap().to_string().clone()),
        Some(lua::Type::Table) => {
            l.pushvalue(offset);
            l.pushnil();
            let mut d: BTreeMap<String, JsonValue> = BTreeMap::new();

            while l.next(-2) {
                // Keys could be strings or numbers, depending on whether it's a map-shaped table
                // or an array-shaped table. We can't rely on `l.tostring` because we're in the
                // middle of a next() loop.
                let k = match l.type_(-2) {
                    Some(lua::Type::String) => {
                        l.checkstring(-2).unwrap().to_string().clone()
                    },
                    Some(lua::Type::Number) => {
                        l.checknumber(-2).to_string()
                    },
                    k_type => {
                        panic!("Unknown key type: {:?}", k_type);
                    }
                };

                let v: JsonValue = try!(deserialize_json(l, -1));
                d.insert(k, v);
                l.pop(1);
            }

            l.pop(1);

            JsonValue::Object(d)
        },
        _ => {
            return Err(LuaError::Generic("Could not deserialize return value".to_string()))
        }
    })
}

pub unsafe fn serialize_json(l: &mut lua::ExternState, json: JsonValue) {
    match json {
        JsonValue::Null => l.pushnil(),
        JsonValue::Bool(v) => l.pushboolean(v),
        JsonValue::I64(v) => l.pushstring(&v.to_string()[..]),
        JsonValue::U64(v) => l.pushstring(&v.to_string()[..]),
        JsonValue::F64(v) => l.pushnumber(v),
        JsonValue::String(v) => l.pushstring(&v[..]),
        JsonValue::Array(v) => {
            l.newtable();

            for (i, iv) in v.iter().enumerate() {
                l.pushinteger((i + 1) as isize);
                serialize_json(l, iv.clone());
                l.settable(-3);
            }
        },
        JsonValue::Object(v) => {
            l.newtable();

            for (k, iv) in &v {
                l.pushstring(k);
                serialize_json(l, iv.clone());
                l.settable(-3);
            }
        }
    }
}

pub unsafe fn serialize_edges(l: &mut lua::ExternState, edges: Vec<Edge<Uuid>>) {
    l.newtable();

    for (i, edge) in edges.iter().enumerate() {
        l.pushinteger((i + 1) as isize);
        serialize_edge(l, &edge);
        l.settable(-3);
    }
}

pub unsafe fn serialize_edge(l: &mut lua::ExternState, edge: &Edge<Uuid>) {
    l.newtable();
    add_string_field_to_table(l, "outbound_id", &edge.outbound_id.to_string()[..]);
    add_string_field_to_table(l, "type", &edge.t.0[..]);
    add_string_field_to_table(l, "inbound_id", &edge.inbound_id.to_string()[..]);
    add_number_field_to_table(l, "weight", edge.weight.0 as f64);
}

pub unsafe fn add_string_field_to_table(l: &mut lua::ExternState, k: &str, v: &str) {
    l.pushstring(v);
    l.setfield(-2, k);
}

pub unsafe fn add_number_field_to_table(l: &mut lua::ExternState, k: &str, v: f64) {
    l.pushnumber(v);
    l.setfield(-2, k);
}

pub unsafe fn get_string_param(l: &mut lua::ExternState, narg: i32) -> Result<String, LuaError> {
    match l.checkstring(narg) {
        Some(s) => Ok(s.to_string()),
        None => Err(LuaError::Arg(narg, "Expected string".to_string()))
    }
}

pub unsafe fn get_type_param(l: &mut lua::ExternState, narg: i32) -> Result<Type, LuaError> {
    let s = try!(get_string_param(l, narg));
    Ok(try!(Type::new(s)))
}

pub unsafe fn get_optional_i64_param(l: &mut lua::ExternState, narg: i32) -> Result<Option<i64>, LuaError> {
    let s = try!(get_string_param(l, narg));

    if s == "" {
        return Ok(None);
    }

    match i64::from_str_radix(&s[..], 10) {
        Ok(i) => Ok(Some(i)),
        Err(_) => Err(LuaError::Arg(narg, "Expected i64 as string".to_string()))
    }
}

pub unsafe fn get_uuid_param(l: &mut lua::ExternState, narg: i32) -> Result<Uuid, LuaError> {
    let s = try!(get_string_param(l, narg));

    match Uuid::from_str(&s[..]) {
        Ok(u) => Ok(u),
        Err(_) => Err(LuaError::Arg(narg, "Expected uuid as string".to_string()))
    }
}

pub unsafe fn get_optional_datetime_param(l: &mut lua::ExternState, narg: i32) -> Result<Option<NaiveDateTime>, LuaError> {
    match try!(get_optional_i64_param(l, narg)) {
        Some(i) => Ok(Some(NaiveDateTime::from_timestamp(i, 0))),
        None => Ok(None)
    }
}

pub unsafe fn get_limit_param(l: &mut lua::ExternState, narg: i32) -> Result<u16, LuaError> {
    match l.checkinteger(narg) {
        i if i > u16::MAX as isize => Ok(u16::MAX),
        i if i < 0 => Err(LuaError::Arg(narg, "Limit cannot be negative".to_string())),
        i => Ok(i as u16)
    }
}

pub unsafe fn get_offset_param(l: &mut lua::ExternState, narg: i32) -> Result<u64, LuaError> {
    match l.checkinteger(narg) {
        i if i < 0 => return Err(LuaError::Arg(3, "Offset cannot be negative".to_string())),
        i => Ok(i as u64)
    }
}

pub unsafe fn get_weight_param(l: &mut lua::ExternState, narg: i32) -> Result<Weight, LuaError> {
    let w = l.checknumber(narg);
    Ok(try!(Weight::new(w as f32)))
}

pub unsafe fn serialize_u64(l: &mut lua::ExternState, val: u64) {
    l.pushinteger(match val {
        i if i > isize::MAX as u64 => isize::MAX,
        i => i as isize
    })
}