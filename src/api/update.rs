use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
#[cache(valid_for = 30)]
pub async fn get(pool: &'static Pool, user_id: i64, plant_id: i64) -> Result<Update> {
    // TODO: we currently don't allow global updates, but we should (at least by groups)
    let last_update: Update = sqlx::query_as(
        "SELECT id, plant_id, file_hash, file_name, created_at
        FROM updates
        WHERE user_id = $1 AND plant_id = $2
        ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(plant_id)
    .fetch_one(pool)
    .await?;
    Ok(last_update)
}

#[exec_time]
pub async fn new(
    pool: &'static Pool,
    user_id: i64,
    plant_id: i64,
    file_hash: String,
    file_name: String,
) -> Result<()> {
    // TODO: Redundant since we have plant_id?
    api::plant::owns(pool, user_id, plant_id).await?;
    sqlx::query(
        "INSERT INTO updates (plant_id, owner_id, file_hash, file_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(plant_id)
    .bind(user_id)
    .bind(&file_hash)
    .bind(&file_name)
    .execute(pool)
    .await?;
    Ok(())
}
