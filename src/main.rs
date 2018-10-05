extern crate pretty_env_logger;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate warp;

use std::env;
use std::sync::{Arc, Mutex};
use warp::{http::StatusCode, Filter};

fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "api_log=info");
    }
    pretty_env_logger::init();

    let hello = path!("hello" / String)
        .map(|name| format!("Hello, {}!", name));

    let api = hello.with(warp::log("api_log"));

    warp::serve(api)
        .run(([127, 0, 0, 1], 3030));
}
