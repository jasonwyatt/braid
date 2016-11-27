#[macro_use] mod macros;
mod api;
mod errors;
mod util;

use lua;
use libc;
use serde_json::value::Value as JsonValue;
use common::ProxyTransaction;
use nutrino::Transaction;
use uuid::Uuid;
use self::errors::ScriptError;

pub fn run(mut trans: ProxyTransaction, account_id: Uuid, source: &str, arg: JsonValue) -> Result<JsonValue, ScriptError> {
    let mut l = lua::State::new();
    l.openlibs();

    l.register("get_vertex", api::get_vertex);
    l.register("create_vertex", api::create_vertex);
    l.register("set_vertex", api::set_vertex);
    l.register("delete_vertex", api::delete_vertex);

    l.register("get_edge", api::get_edge);
    l.register("set_edge", api::set_edge);
    l.register("delete_edge", api::delete_edge);
    
    l.register("get_edge_count", api::get_edge_count);
    l.register("get_edge_range", api::get_edge_range);
    l.register("get_edge_time_range", api::get_edge_time_range);

    l.register("get_reversed_edge_count", api::get_reversed_edge_count);
    l.register("get_reversed_edge_range", api::get_reversed_edge_range);
    l.register("get_reversed_edge_time_range", api::get_reversed_edge_time_range);

    l.register("get_global_metadata", api::get_global_metadata);
    l.register("set_global_metadata", api::set_global_metadata);
    l.register("delete_global_metadata", api::delete_global_metadata);
    l.register("get_account_metadata", api::get_account_metadata);
    l.register("set_account_metadata", api::set_account_metadata);
    l.register("delete_account_metadata", api::delete_account_metadata);
    l.register("get_vertex_metadata", api::get_vertex_metadata);
    l.register("set_vertex_metadata", api::set_vertex_metadata);
    l.register("delete_vertex_metadata", api::delete_vertex_metadata);
    l.register("get_edge_metadata", api::get_edge_metadata);
    l.register("set_edge_metadata", api::set_edge_metadata);
    l.register("delete_edge_metadata", api::delete_edge_metadata);

    if let Err(err) = l.loadstring(source) {
        return Err(ScriptError::new_from_loaderror(&mut l, err));
    }

    let trans_ptr: *mut libc::c_void = &mut trans as *mut _ as *mut libc::c_void;

    l.pushlightuserdata(trans_ptr);
    l.setglobal("trans");

    l.pushstring(&account_id.to_string()[..]);
    l.setglobal("account_id");

    unsafe {
        util::serialize_json(l.as_extern(), arg);
    }

    l.setglobal("arg");

    if let Err(err) = l.pcall(0, lua::MULTRET, 0) {
        return Err(ScriptError::new_from_pcallerror(&mut l, err));
    }

    if let Err(err) = trans.commit() {
        return Err(ScriptError::Runtime(format!("Could not commit script transaction: {}", err)))
    }

    if l.gettop() == 0 {
        Ok(JsonValue::Null)
    } else {
        unsafe {
            Ok(try!(util::deserialize_json(l.as_extern(), -1)))
        }
    }
}