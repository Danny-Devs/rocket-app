#[macro_use]
extern crate rocket;

mod auth;
mod models;
mod repositories;
mod schema;

use auth::BasicAuth;
use diesel::result::Error::NotFound;
use models::{NewRustacean, Rustacean};
use repositories::RustaceanRepository;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::response::status;
use rocket::response::status::Custom;
use rocket::serde::json::{json, Json, Value};
use rocket::{Build, Rocket};
use rocket_sync_db_pools::database;

#[database("sqlite")]
#[allow(dead_code)]
struct DbConn(diesel::SqliteConnection);

#[get("/rustaceans")]
async fn get_rustaceans(_auth: BasicAuth, db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(|c| {
        RustaceanRepository::find_multiple(c, 1000)
            .map(|rustaceans| json!(rustaceans))
            .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string())))
    })
    .await
}

#[get("/rustaceans/<id>")]
async fn view_rustacean(id: i32, _auth: BasicAuth, db: DbConn) -> Result<Value, Custom<Value>> {
    db.run(move |c| {
        RustaceanRepository::find(c, id)
            .map(|rustacean| json!(rustacean))
            .map_err(|e| match e {
                NotFound => Custom(Status::NotFound, json!(e.to_string())),
                _ => Custom(Status::InternalServerError, json!(e.to_string())),
            })
    })
    .await
}

#[post("/rustaceans", format = "json", data = "<new_rustacean>")]
async fn create_rustacean(
    _auth: BasicAuth,
    db: DbConn,
    new_rustacean: Json<NewRustacean>,
) -> Result<Value, Custom<Value>> {
    db.run(|c| {
        RustaceanRepository::create(c, new_rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string())))
    })
    .await
}

#[put("/rustaceans/<id>", format = "json", data = "<rustacean>")]
async fn update_rustacean(
    id: i32,
    _auth: BasicAuth,
    db: DbConn,
    rustacean: Json<Rustacean>,
) -> Result<Value, Custom<Value>> {
    db.run(move |c| {
        if RustaceanRepository::find(c, id).is_err() {
            return Err(Custom(Status::NotFound, json!("Rustacean not found")));
        }

        RustaceanRepository::update(c, id, rustacean.into_inner())
            .map(|rustacean| json!(rustacean))
            .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string())))
    })
    .await
}

#[delete("/rustaceans/<id>")]
async fn delete_rustacean(
    id: i32,
    _auth: BasicAuth,
    db: DbConn,
) -> Result<status::NoContent, Custom<Value>> {
    db.run(move |c| {
        if RustaceanRepository::find(c, id).is_err() {
            return Err(Custom(Status::NotFound, json!("Rustacean not found")));
        }

        RustaceanRepository::delete(c, id)
            .map(|_| status::NoContent)
            .map_err(|e| Custom(Status::InternalServerError, json!(e.to_string())))
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

async fn run_db_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
    DbConn::get_one(&rocket)
        .await
        .expect("Unable to retrieve connection")
        .run(|c| {
            c.run_pending_migrations(MIGRATIONS)
                .expect("Failed to run migrations");
        })
        .await;

    rocket
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
        .attach(AdHoc::on_ignite("Diesel migrations", run_db_migrations))
        .launch()
        .await;
}
