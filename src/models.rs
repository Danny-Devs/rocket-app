use crate::schema::rustaceans;
use diesel::{Insertable, Queryable, AsChangeset};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Queryable, AsChangeset)]
#[diesel(table_name = rustaceans)]
pub struct Rustacean {
    #[serde(skip_deserializing)]
    pub id: i32,
    pub name: String,
    pub email: String,
    #[serde(skip_deserializing)]
    pub created_at: String,
}

#[derive(Deserialize, Insertable)]
#[diesel(table_name = rustaceans)]
pub struct NewRustacean {
    pub name: String,
    pub email: String,
}
