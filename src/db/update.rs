use crate::db::device::DeviceId;
use crate::db::user::UserId;
use crate::prelude::*;
use codegen::{cache, exec_time};

#[exec_time]
//#[cache(valid_for = 30)]
pub async fn get(
    txn: &mut Transaction<'_>,
    user_id: UserId,
    device_id: DeviceId,
) -> Result<Option<Update>> {
    // TODO: we currently don't allow global updates, but we should (at least by groups)
    let last_update: Option<Update> = sqlx::query_as(
        "SELECT id, device_id, file_hash, file_name, version, created_at
        FROM binary_updates
        WHERE owner_id = $1 AND device_id = $2
        ORDER BY created_at DESC",
    )
    .bind(user_id)
    .bind(device_id)
    .fetch_optional(txn)
    .await?;
    Ok(last_update)
}

#[exec_time]
pub async fn new(
    txn: &mut Transaction<'_>,
    user_id: UserId,
    device_id: DeviceId,
    file_hash: String,
    file_name: String,
    version: String,
) -> Result<()> {
    // TODO: Redundant since we have device_id?
    db::plant::owns(txn, user_id, device_id).await?;
    sqlx::query(
        "INSERT INTO binary_updates (device_id, owner_id, file_hash, file_name, version) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(device_id)
    .bind(user_id)
    .bind(&file_hash)
    .bind(&file_name)
    .bind(&version)
    .execute(txn)
    .await?;
    Ok(())
}
