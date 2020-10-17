use crate::prelude::*;
use codegen::{exec_time, cache};

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, event: NewEvent) -> Result<()> {
    api::plant::owns(pool, user_id, event.plant_id).await?;
    sqlx::query("INSERT INTO events (air_temperature_celsius, air_humidity_percentage, soil_resistivity_raw, soil_temperature_celsius, plant_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(event.air_temperature_celsius)
        .bind(event.air_humidity_percentage)
        .bind(event.soil_resistivity_raw)
        .bind(event.soil_temperature_celsius)
        .bind(event.plant_id)
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
