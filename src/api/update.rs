use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
#[cache(valid_for = 30)]
pub async fn get(pool: &'static Pool, user_id: i64, plant_id: i64) -> Result<Option<Update>> {
    // TODO: we currently don't allow global updates, but we should (at least by groups)
    let last_update: Option<Update> = sqlx::query_as(
        "SELECT id, plant_id, file_hash, file_name, version, created_at
        FROM updates
        WHERE owner_id = $1 AND plant_id = $2
        ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(plant_id)
    .fetch_optional(pool)
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
    version: String,
) -> Result<()> {
    // TODO: Redundant since we have plant_id?
    api::plant::owns(pool, user_id, plant_id).await?;
    sqlx::query(
        "INSERT INTO updates (plant_id, owner_id, file_hash, file_name, version) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(plant_id)
    .bind(user_id)
    .bind(&file_hash)
    .bind(&file_name)
    .bind(&version)
    .execute(pool)
    .await?;
    Ok(())
}
