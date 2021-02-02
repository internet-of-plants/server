use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, event: NewEvent) -> Result<()> {
    let plant_id = api::plant::put(pool, user_id, event.mac).await?;
    // TODO: log error if something is NaN
    sqlx::query("INSERT INTO events (air_temperature_celsius, air_humidity_percentage, air_heat_index_celsius, soil_resistivity_raw, soil_temperature_celsius, plant_id) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(event.air_temperature_celsius)
        .bind(event.air_humidity_percentage)
        .bind(event.air_heat_index_celsius)
        .bind(event.soil_resistivity_raw)
        .bind(event.soil_temperature_celsius)
        .bind(plant_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[exec_time]
#[cache(valid_for = 30)]
pub async fn last_event(pool: &'static Pool, plant_id: i64) -> Result<Option<Event>> {
    let event: Option<Event> = sqlx::query_as(
        "SELECT id, air_temperature_celsius, air_humidity_percentage, soil_resistivity_raw, soil_temperature_celsius, plant_id, created_at
        FROM events
        WHERE plant_id = $1
        ORDER BY created_at DESC")
        .bind(plant_id)
        .fetch_optional(pool)
        .await?;
    Ok(event)
}
