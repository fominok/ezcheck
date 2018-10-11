extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate warp;
extern crate serde_json;
#[macro_use]
extern crate log;

use std::env;
use std::sync::{Arc, Mutex};
use warp::{http::StatusCode, Filter};
use std::fs::File;

#[derive(Deserialize, Serialize)]
enum PermissionValueType {
    String,
    Bool,
    Dict,
}

#[derive(Deserialize, Serialize)]
struct Permission {
    name: String,
    value_type: PermissionValueType,
    multiple: bool,
}

#[derive(Deserialize, Serialize, PartialEq)]
struct PermissionValue {
    permission_name: String,
    value: Option<String>,
    user: String,
    app: String,
}

#[derive(Deserialize, Serialize)]
struct DbStruct {
    perms: Vec<Permission>,
    perm_vals: Vec<PermissionValue>,
}

#[derive(Deserialize, Serialize)]
struct CheckResponse {
    permit: bool,
}

type Db = Arc<Mutex<DbStruct>>;

fn read_defaults_to_db(filename: &str, db: Db) {
    if let Ok(file) = File::open(filename) {
        if let Ok(defaults) = serde_json::from_reader(file) {
            let mut db_val = db.lock().unwrap();
            *db_val = defaults;
            info!("Permissions are loaded from {}", filename);
        } else {
            warn!("{} found found, but format is wrong", filename);
        }
    } else {
        warn!("{} not found, starting with empty Db", filename);
    }
}


fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let db = Arc::new(Mutex::new(DbStruct {
        perms: Vec::new(),
        perm_vals: Vec::new(),
    }));

    read_defaults_to_db("defaults.json", db.clone());

    let db_filt = warp::any().map(move || db.clone());

    let permissions = warp::path("permissions");
    let permission_values = warp::path("permission_values");

    let permissions_index = permissions.and(warp::path::index());
    let permission_values_index = permission_values.and(warp::path::index());

    let list_permissions = warp::get2()
        .and(permissions_index)
        .and(db_filt.clone())
        .map(list_permissions);

    let create_permission = warp::post2()
        .and(permissions_index)
        .and(warp::body::json())
        .and(db_filt.clone())
        .map(create_permission);

    let list_permission_values = warp::get2()
        .and(permission_values_index)
        .and(db_filt.clone())
        .map(list_permission_values);

    let create_permission_value = warp::post2()
        .and(permission_values_index)
        .and(warp::body::json())
        .and(db_filt.clone())
        .and_then(create_permission_value);

    let permission_check = path!("permission_check")
        .and(warp::query::query())
        .and(db_filt.clone())
        .map(permission_check);

    let api = list_permissions
        .or(create_permission_value)
        .or(list_permission_values)
        .or(create_permission)
        .or(permission_check)
        .with(warp::log("api_log"));

    warp::serve(api).run(([127, 0, 0, 1], 3030));
}

fn list_permissions(db: Db) -> impl warp::Reply {
    warp::reply::json(&db.lock().unwrap().perms)
}

fn create_permission(create: Permission, db: Db) -> impl warp::Reply {
    db.lock().unwrap().perms.push(create);
    Ok(StatusCode::CREATED)
}

fn list_permission_values(db: Db) -> impl warp::Reply {
    warp::reply::json(&db.lock().unwrap().perm_vals)
}

fn create_permission_value(
    create: PermissionValue,
    db: Db,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db_ref = db.lock().unwrap();
    if let Some(_) = db_ref
        .perms
        .iter_mut()
        .find(|p| p.name == create.permission_name)
    {
        db_ref.perm_vals.push(create);
        Ok(StatusCode::CREATED)
    } else {
        Err(warp::reject::reject())
    }
}

fn permission_check(check: PermissionValue, db: Db) -> impl warp::Reply {
    let resp = db
        .lock()
        .unwrap()
        .perm_vals
        .iter()
        .find(|&p| p == &check)
        .is_some();
    warp::reply::json(&CheckResponse { permit: resp })
}
