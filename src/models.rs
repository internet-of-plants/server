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
    pub mac: Option<String>,
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
    #[serde(with = "crate::utils::string")]
    pub id: i64,
    pub name: String,
    pub mac_address: String,
    pub description: Option<String>,
    pub owner_id: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct NewEvent {
    #[serde(with = "crate::utils::float")]
    pub air_temperature_celsius: f32,
    #[serde(with = "crate::utils::float")]
    pub air_humidity_percentage: f32,
    #[serde(with = "crate::utils::float")]
    pub air_heat_index_celsius: f32,
    pub soil_resistivity_raw: i16,
    #[serde(with = "crate::utils::float")]
    pub soil_temperature_celsius: f32,
    pub firmware_hash: String,
    pub mac: String,
}

#[derive(FromRow, Debug, Clone, PartialEq, Serialize)]
pub struct Event {
    pub id: i64,
    pub air_temperature_celsius: f32,
    pub air_humidity_percentage: f32,
    pub air_heat_index_celsius: f32,
    pub soil_resistivity_raw: i16,
    pub soil_temperature_celsius: f32,
    #[serde(with = "crate::utils::string")]
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
    pub since_secs_ago: u64,
}

#[derive(FromRow, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Update {
    pub id: i64,
    #[serde(with = "crate::utils::maybe_string")]
    pub plant_id: Option<i64>,
    pub file_hash: String,
    pub file_name: String,
    pub created_at: i64,
}

#[derive(FromRow, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeviceLog {
    pub id: i64,
    pub plant_id: i64,
    pub log: String,
    pub created_at: i64,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewDevicePanic {
    pub mac: String,
    pub file: String,
    pub line: u32,
    pub func: String,
    pub msg: String,
}

#[derive(FromRow, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DevicePanic {
    pub id: i64,
    pub plant_id: i64,
    pub file: String,
    pub line: u32,
    pub func: String,
    pub msg: String,
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
