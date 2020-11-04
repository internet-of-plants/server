use crate::prelude::*;
use codegen::{cache, exec_time};
use std::time::Duration;

#[exec_time]
#[cache]
pub async fn get_plain(pool: &'static Pool, plant_id: i64) -> Result<Plant> {
    let plant: Option<Plant> = sqlx::query_as(
        "SELECT id, name, description, owner_id, created_at
        FROM plants
        WHERE id = $1")
        .bind(plant_id)
        .fetch_optional(pool)
        .await?;
    match plant {
        Some(plant) => Ok(plant),
        None => Err(Error::NothingFound),
    }
}

#[exec_time]
pub async fn get(pool: &'static Pool, user_id: i64, plant_id: i64) -> Result<Status> {
    api::plant::owns(pool, user_id, plant_id).await?;
    let plant = api::plant::get_plain(pool, plant_id).await?;
    let event = api::event::last_event(pool, plant_id).await?;
    let now = api::now(pool).await?;
    Ok(Status { plant, event, now })
}

#[exec_time]
#[cache(valid_for = 3600)]
pub async fn history(pool: &'static Pool, user_id: i64, plant_id: i64, since: Duration) -> Result<StatusHistory> {
    api::plant::owns(pool, user_id, plant_id).await?;
    let plant = api::plant::get_plain(pool, plant_id).await?;
    let now = api::now(pool).await?;

    let since = now - since.as_secs();
    let events: Vec<Event> = sqlx::query_as(
        "SELECT id, air_temperature_celsius, air_humidity_percentage, soil_resistivity_raw, soil_temperature_celsius, plant_id, created_at
        FROM events
        WHERE plant_id = $1
              AND created_at > $2
        ORDER BY created_at ASC")
        .bind(plant_id)
        .bind(since as i64)
        .fetch_all(pool)
        .await?;
    Ok(StatusHistory { plant, events, now })
}

#[exec_time]
pub async fn index(pool: &'static Pool, user_id: i64) -> Result<Vec<Status>> {
    let now = api::now(pool).await?;
    let plants: Vec<Id> = sqlx::query_as(
        "SELECT id
        FROM plants
        WHERE owner_id = $1
        ORDER BY created_at DESC")
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    let mut status = Vec::with_capacity(plants.len());
    for Id { id } in plants {
        let event = api::event::last_event(pool, id).await?;
        let plant = api::plant::get_plain(pool, id).await?;
        status.push(Status { plant, event, now })
    }
    Ok(status)
}

#[exec_time]
#[cache]
pub async fn owns(pool: &'static Pool, user_id: i64, plant_id: i64) -> Result<()> {
    let exists: Option<(i32,)> = sqlx::query_as(
        "SELECT 1
        FROM plants
        WHERE owner_id = $1
              AND id = $2")
        .bind(user_id)
        .bind(plant_id)
        .fetch_optional(pool)
        .await?;
    match exists {
        Some(_) => Ok(()),
        None => Err(Error::NothingFound),
    }
}

#[exec_time]
pub async fn put(pool: &'static Pool, user_id: i64, mac_address: String) -> Result<i64> {
    let plant: Option<Id> = sqlx::query_as("SELECT ip
                                            FROM plants
                                            WHERE mac = $1
                                                  AND owner_id = $2")
        .bind(&mac_address)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

    if let Some(Id { id }) = plant {
        return Ok(id);
    }

    let name = utils::random_name();
    // This can theoretically conflict, but it will just return 500
    // So in the one in a million time this happens the client just retries
    let id: i64 = i64::saturating_abs(rand::random());
    sqlx::query("INSERT INTO plants (id, mac, name, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(&mac_address)
        .bind(&name)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(id)
}
