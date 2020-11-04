use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

pub type Pool = PgPool;

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub username: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Login {
    pub email: String,
    pub password: String,
}

#[derive(FromRow, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct User {
    pub email: String,
    #[serde(skip)]
    pub password_hash: String,
    pub username: String,
    pub created_at: i64,
}

#[derive(FromRow, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plant {
    // i64 isn't really supported at json level, since it's actually f64
    // we generally don't care, but here we set id with rand::random, so sometimes
    // it breaks the json representation as a number
    #[serde(with  = "crate::utils::string")]
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: i64,
    pub created_at: i64,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
pub struct NewEvent {
    pub air_temperature_celsius: f32,
    pub air_humidity_percentage: f32,
    pub air_heat_index_celsius: f32,
    pub soil_resistivity_raw: i16,
    pub soil_temperature_celsius: f32,
    #[serde(with  = "crate::utils::string")]
    pub plant_id: i64,
}

#[derive(FromRow, Debug, Clone, PartialEq, Serialize)]
pub struct Event {
    pub id: i64,
    pub air_temperature_celsius: f32,
    pub air_humidity_percentage: f32,
    pub air_heat_index_celsius: f32,
    pub soil_resistivity_raw: i16,
    pub soil_temperature_celsius: f32,
    #[serde(with  = "crate::utils::string")]
    pub plant_id: i64,
    pub created_at: i64,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Mac {
    pub mac: String,
}

#[derive(FromRow, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Id {
    pub id: i64,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RequestHistory {
    pub id: i64,
    pub since_secs_ago: u64
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ErrorReport {
    pub id: i64,
    #[serde(with  = "crate::utils::string")]
    pub plant_id: i64,
    pub error: String,
    pub is_solved: bool,
    pub created_at: i64,
}

#[derive(FromRow, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ErrorDump {
    pub id: i64,
    #[serde(with  = "crate::utils::string")]
    pub plant_id: i64,
    pub error: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Status {
    pub plant: Plant,
    pub event: Option<Event>,
    pub now: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatusHistory {
    pub plant: Plant,
    pub now: u64,
    pub events: Vec<Event>,
}

#[derive(FromRow, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Migration {
    pub id: i16,
}
