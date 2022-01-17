use crate::db::device::DeviceId;
use crate::db::user::UserId;
use crate::prelude::*;
use codegen::{cache, exec_time};
use std::time::Duration;

#[exec_time]
//#[cache]
pub async fn get_plain(txn: &mut Transaction<'_>, plant_id: DeviceId) -> Result<Plant> {
    let plant: Option<Plant> = sqlx::query_as(
        "SELECT p.id, p.name, p.mac, u.version, u.file_hash, p.description, p.owner_id, p.created_at
        FROM plants p
        LEFT JOIN updates u ON u.plant_id = p.id
        WHERE p.id = $1
        ORDER BY u.created_at DESC
        LIMIT 1",
    )
    .bind(plant_id)
    .fetch_optional(&mut *txn)
    .await?;
    match plant {
        Some(plant) => Ok(plant),
        None => Err(Error::NothingFound),
    }
}

#[exec_time]
pub async fn get(txn: &mut Transaction<'_>, user_id: UserId, plant_id: DeviceId) -> Result<Status> {
    db::plant::owns(txn, user_id, plant_id).await?;
    let plant = db::plant::get_plain(txn, plant_id).await?;
    let event = db::event::last_event(txn, plant_id).await?;
    let now = db::now(&mut *txn).await?;
    Ok(Status { plant, event, now })
}

#[exec_time]
//#[cache(valid_for = 3600)]
pub async fn history(
    txn: &mut Transaction<'_>,
    user_id: UserId,
    plant_id: DeviceId,
    since: Duration,
) -> Result<StatusHistory> {
    db::plant::owns(txn, user_id, plant_id).await?;
    let plant = db::plant::get_plain(txn, plant_id).await?;
    let now = db::now(&mut *txn).await?;

    let since = now - since.as_secs();
    let events: Vec<Event> = sqlx::query_as(
        "SELECT id, air_temperature_celsius, air_humidity_percentage, air_heat_index_celsius, soil_resistivity_raw, soil_temperature_celsius, plant_id, hash, created_at
        FROM events
        WHERE plant_id = $1
              AND created_at > $2
        ORDER BY created_at ASC")
        .bind(plant_id)
        .bind(since as i64)
        .fetch_all(&mut *txn)
        .await?;
    Ok(StatusHistory { plant, events, now })
}

#[exec_time]
//#[cache(valid_for = 30)]
pub async fn index(txn: &mut Transaction<'_>, user_id: UserId) -> Result<Vec<Status>> {
    let now = db::now(&mut *txn).await?;
    let plants: Vec<(DeviceId,)> = sqlx::query_as(
        "SELECT id
        FROM plants
        WHERE owner_id = $1
        ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&mut *txn)
    .await?;
    let mut status = Vec::with_capacity(plants.len());
    for (id,) in plants {
        let event = db::event::last_event(txn, id).await?;
        let plant = db::plant::get_plain(txn, id).await?;
        status.push(Status { plant, event, now })
    }
    Ok(status)
}

#[exec_time]
//#[cache(valid_for = 30)]
pub async fn owns(txn: &mut Transaction<'_>, user_id: UserId, plant_id: DeviceId) -> Result<()> {
    let exists: Option<(i32,)> = sqlx::query_as(
        "SELECT 1
        FROM plants
        WHERE owner_id = $1
              AND id = $2",
    )
    .bind(user_id)
    .bind(plant_id)
    .fetch_optional(&mut *txn)
    .await?;
    match exists {
        Some(_) => Ok(()),
        None => Err(Error::NothingFound),
    }
}

#[exec_time]
pub async fn put(txn: &mut Transaction<'_>, user_id: UserId, mac_address: String) -> Result<i64> {
    let plant: Option<Id> = sqlx::query_as(
        "SELECT id
        FROM plants
        WHERE mac = $1
        AND owner_id = $2",
    )
    .bind(&mac_address)
    .bind(user_id)
    .fetch_optional(&mut *txn)
    .await?;

    if let Some(Id { id }) = plant {
        return Ok(id);
    }

    // This can theoretically conflict, but it will just return 500
    // So in the one in a million time this happens the client just retries
    // Id must be unique, name must be unique to a user
    let name = utils::random_name();
    let id: i64 = i64::saturating_abs(rand::random());
    sqlx::query("INSERT INTO plants (id, mac, name, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(&mac_address)
        .bind(&name)
        .bind(user_id)
        .execute(&mut *txn)
        .await?;
    Ok(id)
}
