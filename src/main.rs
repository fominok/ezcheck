extern crate pretty_env_logger;
extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate warp;

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

#[derive(Deserialize, Serialize)]
struct CheckResponse {
    exist: bool
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


    let api = list_permissions
        .or(create_permission_value)
        .or(list_permission_values)
        .or(create_permission)
        .with(warp::log("api_log"));

    warp::serve(api)
        .run(([127, 0, 0, 1], 3030));
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
fn create_permission_value(create: PermissionValue, db: Db)
                           -> Result<impl warp::Reply, warp::Rejection> {
    let mut db_ref = db.lock().unwrap();
    if let Some(_) = db_ref.perms.iter_mut()
        .find(|p| p.name == create.permission_name) {
            db_ref.perm_vals.push(create);
            Ok(StatusCode::CREATED)
        } else {
            Err(warp::reject::reject())
        }
}
