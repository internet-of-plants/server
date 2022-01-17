use crate::db::device::DeviceId;
use crate::db::user::UserId;
use derive_more::FromStr;
use crate::prelude::*;
use codegen::{cache, exec_time};
use serde::{Deserialize, Serialize};
use crate::db::timestamp::{now, DateTime};

pub async fn last_event(txn: &mut Transaction<'_>, farm_id: DeviceId) -> Result<Option<crate::models::Event>> {
    let event: Option<crate::models::Event> = sqlx::query_as(
        "SELECT id, air_temperature_celsius, air_humidity_percentage, air_heat_index_celsius, soil_resistivity_raw, soil_temperature_celsius, farm_id, hash, created_at
        FROM events
        WHERE farm_id = $1
        ORDER BY created_at DESC")
        .bind(farm_id)
        .fetch_optional(txn)
        .await?;
    debug!("Last Event: {:?}", event);
    Ok(event)
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct EventId(i64);

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Event {
    id: EventId,
    air_temperature_celsius: f64,
    air_humidity_percentage: f64,
    air_heat_index_celsius: f64,
    soil_resistivity_raw: i16,
    soil_temperature_celsius: f64,
    firmware_hash: String,
    created_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NewEvent {
    air_temperature_celsius: f64,
    air_humidity_percentage: f64,
    air_heat_index_celsius: f64,
    soil_resistivity_raw: i16,
    soil_temperature_celsius: f64,
}

impl Event {
    pub fn id(&self) -> &EventId {
        &self.id
    }

    pub async fn new(txn: &mut Transaction<'_>, device_id: &DeviceId, new_event: NewEvent, file_hash: String) -> Result<Self> {
        //db::plant::owns(txn, user_id, farm_id).await?;
        // TODO: log error if something is NaN
        let (event_id,): (EventId,) = sqlx::query_as("INSERT INTO events (device_id) VALUES ($1) RETURNING id")
            .bind(device_id)
            .fetch_one(&mut *txn)
            .await?;
        sqlx::query("INSERT INTO measurements (air_temperature_celsius, air_humidity_percentage, air_heat_index_celsius, soil_resistivity_raw, soil_temperature_celsius, event_id, firmware_hash) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(new_event.air_temperature_celsius)
            .bind(new_event.air_humidity_percentage)
            .bind(new_event.air_heat_index_celsius)
            .bind(new_event.soil_resistivity_raw)
            .bind(new_event.soil_temperature_celsius)
            .bind(event_id)
            .bind(&file_hash)
            .execute(txn)
            .await?;
        Ok(Self {
            id: event_id,
            air_temperature_celsius: new_event.air_temperature_celsius,
            air_humidity_percentage: new_event.air_humidity_percentage,
            air_heat_index_celsius: new_event.air_heat_index_celsius,
            soil_resistivity_raw: new_event.soil_resistivity_raw,
            soil_temperature_celsius: new_event.soil_temperature_celsius,
            firmware_hash: file_hash,
            created_at: now(), // TODO: fix this
        })
    }

    pub async fn last_from_device(txn: &mut Transaction<'_>, device_id: &DeviceId) -> Result<Option<Self>> {
        let event: Option<Event> = sqlx::query_as(
            "SELECT m.id, m.air_temperature_celsius, m.air_humidity_percentage, m.air_heat_index_celsius, m.soil_resistivity_raw, m.soil_temperature_celsius, m.firmware_hash, m.created_at
            FROM measurements as m
            INNER JOIN events as e ON e.id = m.event_id
            WHERE e.device_id = $1
            ORDER BY e.created_at DESC")
            .bind(device_id)
            .fetch_optional(txn)
            .await?;
        debug!("Last Event: {:?}", event);
        Ok(event)
    }
}
