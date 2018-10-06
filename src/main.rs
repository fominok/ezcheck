extern crate pretty_env_logger;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate warp;

use std::env;
use std::sync::{Arc, Mutex};
use warp::{http::StatusCode, Filter};

#[derive(Deserialize, Serialize)]
enum PermissionValueType {
    String,
    Bool,
    Dict
}

#[derive(Deserialize, Serialize)]
struct Permission {
    name: String,
    value_type: PermissionValueType,
    multiple: bool
}

#[derive(Deserialize, Serialize)]
struct PermissionValue {
    permission_name: String,
    value: Option<String>,
    user: String,
    app: String
}

#[derive(Deserialize, Serialize)]
struct DbStruct {
    perms: Vec<Permission>,
    perm_vals: Vec<PermissionValue>
}

type Db = Arc<Mutex<DbStruct>>;

fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "api_log=info");
    }
    pretty_env_logger::init();

    let db = Arc::new(Mutex::new(DbStruct{
        perms: Vec::new(),
        perm_vals: Vec::new()
    }));

    let db_filt = warp::any().map(move || db.clone());

    let permissions = warp::path("permissions");

    let permissions_index = permissions.and(warp::path::index());

    let list = warp::get2()
        .and(permissions_index)
        .and(db_filt.clone())
        .map(list_permissions);

    // let hello = path!("hello" / String)
    //     .map(|name| format!("Hello, {}!", name));

    let api = list.with(warp::log("api_log"));

    warp::serve(api)
        .run(([127, 0, 0, 1], 3030));
}

fn list_permissions(db: Db) -> impl warp::Reply {
    warp::reply::json(&db.lock().unwrap().perms)
}

fn list_permission_values(db: Db) -> impl warp::Reply {
    warp::reply::json(&db.lock().unwrap().perm_vals)
}
