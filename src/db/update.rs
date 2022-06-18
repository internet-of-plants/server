use crate::db::firmware::{Firmware, FirmwareId};
use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::{CollectionId, Device, DeviceId, UserId};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct UpdateId(pub i64);

impl UpdateId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Update {
    id: UpdateId,
    collection_id: CollectionId,
    firmware_id: FirmwareId,
    version: String,
    created_at: DateTime,
}

impl Update {
    pub fn id(&self) -> &UpdateId {
        &self.id
    }

    pub async fn firmware(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        Firmware::find_by_id(txn, self.firmware_id).await
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        _user_id: UserId,
        device_id: DeviceId,
        firmware_id: FirmwareId,
        version: String,
    ) -> Result<Self> {
        let device = Device::find_by_id(txn, &device_id).await?;
        // TODO: Redundant since we have device_id?
        //db::plant::owns(txn, user_id, device_id).await?;
        let (id,) = sqlx::query_as::<_, (UpdateId,)>(
            "INSERT INTO binary_updates (collection_id, firmware_id, version) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(device.collection_id())
            .bind(&firmware_id)
            .bind(&version)
            .fetch_one(txn)
            .await?;
        Ok(Self {
            id,
            collection_id: *device.collection_id(),
            firmware_id,
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
            "SELECT binary_updates.id, binary_updates.collection_id, binary_updates.firmware_id, binary_updates.version, binary_updates.created_at
            FROM binary_updates
            INNER JOIN devices ON devices.collection_id = binary_updates.collection_id
            WHERE devices.id = $1
            ORDER BY binary_updates.created_at DESC",
        )
        .bind(device_id)
        .fetch_optional(txn)
        .await?;
        Ok(last_update)
    }
}
