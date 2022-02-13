use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::{CollectionId, Device, DeviceId, UserId};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct UpdateId(i64);

impl UpdateId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Update {
    id: UpdateId,
    collection_id: CollectionId,
    file_hash: String,
    file_name: String,
    version: String,
    created_at: DateTime,
}

impl Update {
    pub fn id(&self) -> &UpdateId {
        &self.id
    }

    pub fn file_hash(&self) -> &str {
        &self.file_hash
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        _user_id: UserId,
        device_id: DeviceId,
        file_hash: String,
        file_name: String,
        version: String,
    ) -> Result<Self> {
        let device = Device::find_by_id(txn, &device_id).await?;
        // TODO: Redundant since we have device_id?
        //db::plant::owns(txn, user_id, device_id).await?;
        let (id,) = sqlx::query_as::<_, (UpdateId,)>(
            "INSERT INTO binary_updates (collection_id, file_hash, file_name, version) VALUES ($1, $2, $3, $4) RETURNING id",
        )
            .bind(device.collection_id())
            .bind(&file_hash)
            .bind(&file_name)
            .bind(&version)
            .fetch_one(txn)
            .await?;
        Ok(Self {
            id,
            collection_id: *device.collection_id(),
            file_hash,
            file_name,
            version,
            created_at: now(),
        })
    }

    pub async fn find_by_device(
        txn: &mut Transaction<'_>,
        _user_id: UserId,
        device_id: DeviceId,
    ) -> Result<Option<Self>> {
        // TODO: we currently don't allow global updates, but we should (at least by groups)
        let last_update: Option<Update> = sqlx::query_as(
            "SELECT id, collection_id, file_hash, file_name, version, created_at
            FROM binary_updates
            WHERE collection_id = $1
            ORDER BY created_at DESC",
        )
        .bind(device_id)
        .fetch_optional(txn)
        .await?;
        Ok(last_update)
    }
}
