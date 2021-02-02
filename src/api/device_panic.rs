use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
#[cache(valid_for = 30)]
pub async fn index(
    pool: &'static Pool,
    user_id: i64,
    plant_id: Option<i64>,
) -> Result<Vec<DevicePanic>> {
    let device_panics: Vec<DevicePanic> = sqlx::query_as(
        "SELECT id, plant_id, _file, _line, func, msg, created_at
        FROM device_panics
        WHERE is_solved = FALSE AND (owner_id = $1 OR plant_id = $2)
        ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(&plant_id)
    .fetch_all(pool)
    .await?;
    Ok(device_panics)
}

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, device_panic: NewDevicePanic) -> Result<()> {
    let plant_id = api::plant::put(pool, user_id, device_panic.mac).await?;
    sqlx::query("INSERT INTO device_panics (plant_id, owner_id, _file, _line, func, msg) VALUES ($1, $2, $3, $4, $5, $6)")
        .bind(plant_id)
        .bind(user_id)
        .bind(&device_panic.file)
        .bind(device_panic.line)
        .bind(&device_panic.func)
        .bind(&device_panic.msg)
        .execute(pool)
        .await?;
    Ok(())
}

#[exec_time]
pub async fn solve(pool: &'static Pool, user_id: i64, error_id: i64) -> Result<()> {
    api::device_panic::owns(pool, user_id, error_id).await?;
    sqlx::query("UPDATE device_panics SET is_solved = TRUE WHERE id = $1")
        .bind(&error_id)
        .execute(pool)
        .await?;
    Ok(())
}

#[exec_time]
#[cache(valid_for = 3600)]
pub async fn owns(pool: &'static Pool, user_id: i64, error_id: i64) -> Result<()> {
    let exists: Option<(i32,)> = sqlx::query_as(
        "SELECT 1
        FROM device_panics
        WHERE device_panics.owner_id = $1
              AND device_panics.id = $2",
    )
    .bind(user_id)
    .bind(error_id)
    .fetch_optional(pool)
    .await?;
    match exists {
        Some(_) => Ok(()),
        None => Err(Error::NothingFound),
    }
}
