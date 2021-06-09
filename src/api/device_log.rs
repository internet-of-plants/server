use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
#[cache(valid_for = 30)]
pub async fn index(pool: &'static Pool, user_id: i64, plant_id: i64) -> Result<Vec<DeviceLog>> {
    let device_logs: Vec<DeviceLog> = sqlx::query_as(
        "SELECT id, plant_id, log, created_at
        FROM device_logs
        WHERE owner_id = $1 OR plant_id = $2
        ORDER BY created_at DESC
        LIMIT 50",
    )
    .bind(user_id)
    .bind(&plant_id)
    .fetch_all(pool)
    .await?;
    Ok(device_logs)
}

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, plant_id: i64, log: String) -> Result<()> {
    info!("Creating log (user_id: {}, plant_id: {}): {}", user_id, plant_id, log);
    sqlx::query("INSERT INTO device_logs (plant_id, owner_id, log) VALUES ($1, $2, $3)")
        .bind(plant_id)
        .bind(user_id)
        .bind(&log)
        .execute(pool)
        .await?;
    Ok(())
}
