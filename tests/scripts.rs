extern crate braid;
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate chrono;
extern crate rand;
extern crate regex;
extern crate hyper;
extern crate uuid;

mod common;

use std::io::prelude::*;
use std::fs::File;
use uuid::Uuid;
use hyper::client::Client;
use hyper::status::StatusCode;
pub use rand::{thread_rng, Rng};
pub use regex::Regex;
pub use serde_json::Value as JsonValue;
pub use common::{request, create_account, delete_account, response_to_error_message};

lazy_static! {
    static ref OK_EXPECTED_PATTERN: Regex = Regex::new(r"-- ok: (.+)$").unwrap();
}

macro_rules! test_script {
    ($name:ident) => (
        #[test]
        fn $name() {
            let (account_id, secret) = create_account().unwrap();
            run_script(account_id, secret, stringify!($name));
        }
    )
}

fn run_script(account_id: Uuid, secret: String, name: &str) {
    let mut file = File::open(format!("test_scripts/{}.lua", name)).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let client = Client::new();
    let req = request(
        &client,
        8000,
        account_id,
        secret,
        "POST",
        format!("/script/{}.lua", name),
        vec![]
    );
    let mut res = req.send().unwrap();

    let mut payload = String::new();
    res.read_to_string(&mut payload).unwrap();

    if res.status == StatusCode::Ok {
        if let Some(cap) = OK_EXPECTED_PATTERN.captures(&contents[..]) {
            let s = cap.get(1).unwrap().as_str();
            let expected_result: JsonValue = serde_json::from_str(s).unwrap();
            let actual_result: JsonValue = serde_json::from_str(&payload[..]).unwrap();
            assert_eq!(expected_result, actual_result)
        }
    } else {
        panic!("Unexpected status code: {} - payload: {}", res.status, payload)
    }
}

test_script!(get_vertices);
test_script!(account_metadata);
test_script!(create_vertex_bad_type);
test_script!(create_vertex);
test_script!(delete_edges);
test_script!(delete_vertices);
test_script!(edge_metadata);
test_script!(get_edge_count);
test_script!(get_edges_bad_high);
test_script!(get_edges_bad_limit);
test_script!(get_edges_bad_low);
test_script!(get_edges);
test_script!(global_metadata);
test_script!(regression_float_serialization);
test_script!(return_array);
test_script!(return_boolean);
test_script!(return_int);
test_script!(return_nil);
test_script!(return_number);
test_script!(return_obj);
test_script!(return_string);
test_script!(set_and_get_edge);
test_script!(vertex_metadata);
