#[macro_use]
extern crate rocket;

mod auth;
mod models;
mod schema;

use auth::BasicAuth;
use diesel::prelude::*;
use models::{NewRustacean, Rustacean};
use rocket::response::status;
use rocket::serde::json::{json, Json, Value};
use rocket_sync_db_pools::database;
use schema::rustaceans;

#[database("sqlite")]
#[allow(dead_code)]
struct DbConn(diesel::SqliteConnection);

#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, db: DbConn) -> Value {
    db.run(|c| {
        let rustaceans = rustaceans::table
            .order(rustaceans::id.desc())
            .limit(1000)
            .load::<Rustacean>(c)
            .expect("DB error");
        json!(rustaceans)
    })
    .await
}

#[get("/rustaceans/<id>")]
async fn view_rustacean(id: i32, _auth: BasicAuth, db: DbConn) -> Value {
    db.run(move |c| {
        let rustacean = rustaceans::table
            .find(id)
            .get_result::<Rustacean>(c)
            .expect("DB error");
        json!(rustacean)
    })
    .await
}

#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustacean(
    _auth: BasicAuth,
    db: DbConn,
    new_rustacean: Json<NewRustacean>,
) -> Value {
    db.run(|c| {
        let result = diesel::insert_into(rustaceans::table)
            .values(new_rustacean.into_inner())
            .execute(c)
            .expect("DB error when inserting");
        json!(result)
    })
    .await
}

#[put("/rustaceans/<id>", format = "json", data = "<rustacean>")]
async fn update_rustacean(
    id: i32,
    _auth: BasicAuth,
    db: DbConn,
    rustacean: Json<Rustacean>,
) -> Result<Value, status::NotFound<Value>> {
    db.run(move |c| {
        let exists = rustaceans::table
            .find(id)
            .first::<Rustacean>(c)
            .optional()
            .expect("DB error checking rustacean");

        if exists.is_none() {
            return Err(status::NotFound(json!("Rustacean not found")));
        }

        let result = diesel::update(rustaceans::table.find(id))
            .set((
                rustaceans::name.eq(rustacean.name.to_owned()),
                rustaceans::email.eq(rustacean.email.to_owned()),
            ))
            .execute(c)
            .expect("DB error when updating");
        Ok(json!(result))
    })
    .await
}

#[delete("/rustaceans/<id>")]
async fn delete_rustacean(id: i32, _auth: BasicAuth, db: DbConn) -> status::NoContent {
    db.run(move |c| {
        diesel::delete(rustaceans::table.find(id))
            .execute(c)
            .expect("DB error when deleting");
        status::NoContent
    })
    .await
}

#[catch(401)]
fn unauthorized() -> Value {
    json!("Unauthorized")
}

#[catch(404)]
fn not_found() -> Value {
    json!("Not found!")
}

#[catch(422)]
fn unprocessable_entity() -> Value {
    json!("Unprocessable Entity: Invalid input data")
}

#[catch(500)]
fn internal_server_error() -> Value {
    json!("Internal Server Error: Database connection error")
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .mount(
            "/",
            routes![
                get_rustaceans,
                view_rustacean,
                create_rustacean,
                update_rustacean,
                delete_rustacean,
            ],
        )
        .register(
            "/",
            catchers![
                not_found,
                unauthorized,
                unprocessable_entity,
                internal_server_error
            ],
        )
        .attach(DbConn::fairing())
        .launch()
        .await;
}
