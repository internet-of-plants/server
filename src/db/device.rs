use crate::{Collection, CollectionId, Event, Workspace, UserId};
use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct DeviceId(i64);

impl DeviceId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LastUpdate {
    version: String,
    file_hash: String,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DeviceView {
    id: DeviceId,
    name: Option<String>,
    description: Option<String>,
    mac: String,
    file_hash: String,
    last_update: Option<LastUpdate>,
    last_event: Option<Event>,
    created_at: DateTime,
    updated_at: DateTime,
}

impl DeviceView {
    pub async fn find_by_id(txn: &mut Transaction<'_>, device_id: &DeviceId) -> Result<Self> {
        let device = Device::find_by_id(&mut *txn, device_id).await?;
        Ok(DeviceView {
            id: device.id,
            name: device.name,
            description: device.description,
            mac: device.mac,
            file_hash: device.file_hash,
            last_update: None, // TODO
            last_event: Event::last_from_device(&mut *txn, device_id).await?,
            created_at: device.created_at,
            updated_at: device.updated_at,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Device {
    id: DeviceId,
    collection_id: CollectionId,
    name: Option<String>,
    description: Option<String>,
    mac: String,
    file_hash: String,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewDevice {
    pub mac: String,
    pub file_hash: String,
}

impl Device {
    pub fn id(&self) -> &DeviceId {
        &self.id
    }

    pub fn collection_id(&self) -> &CollectionId {
        &self.collection_id
    }

    pub async fn set_file_hash(
        txn: &mut Transaction<'_>,
        device_id: &DeviceId,
        file_hash: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE devices SET file_hash = $1 WHERE id = $2")
            .bind(file_hash)
            .bind(device_id)
            .execute(txn)
            .await?;
        Ok(())
    }

    pub async fn put(
        txn: &mut Transaction<'_>,
        user_id: &UserId,
        new_device: NewDevice,
    ) -> Result<Self> {
        // TODO: we are using too many db roundtrips
        let device = Self::try_find_by_mac(&mut *txn, &new_device.mac).await?;
        if let Some(device) = device {
            if device.file_hash != new_device.file_hash {
                Self::set_file_hash(&mut *txn, device.id(), &new_device.file_hash).await?;
            }
            return Ok(device);
        }

        let workspace = Workspace::default_for_user(&mut *txn, user_id).await?;
        let collection = Collection::new(&mut *txn, new_device.mac.clone(), workspace.id()).await?;

        // TODO: improve number_of_plants default
        let (id,) =
            sqlx::query_as::<_, (DeviceId,)>("INSERT INTO devices (mac, file_hash, number_of_plants, collection_id) VALUES ($1, $2, '1', $3) RETURNING id")
                .bind(&new_device.mac)
                .bind(&new_device.file_hash)
                .bind(collection.id())
                .fetch_one(&mut *txn)
                .await?;
        Ok(Self {
            id,
            collection_id: *collection.id(),
            name: None,
            description: None,
            mac: new_device.mac,
            file_hash: new_device.file_hash,
            created_at: now(), // TODO: fix this
            updated_at: now(), // TODO: fix this
        })
    }

    pub async fn try_find_by_mac(txn: &mut Transaction<'_>, mac: &str) -> Result<Option<Self>> {
        let device: Option<Self> = sqlx::query_as(
            "SELECT dev.id, dev.collection_id, dev.name, dev.description, dev.mac, dev.file_hash, dev.created_at, dev.updated_at
             FROM devices as dev
             WHERE dev.mac = $1",
        )
        .bind(mac)
        .fetch_optional(&mut *txn)
        .await?;
        Ok(device)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, device_id: &DeviceId) -> Result<Self> {
        let device: Self = sqlx::query_as(
            "SELECT dev.id, dev.collection_id, dev.name, dev.description, dev.mac, dev.file_hash, dev.created_at, dev.updated_at
             FROM devices as dev
             WHERE dev.id = $1",
        )
        .bind(device_id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(device)
    }

    pub async fn from_collection(
        txn: &mut Transaction<'_>,
        collection_id: &CollectionId,
    ) -> Result<Vec<Self>> {
        let devices: Vec<Self> = sqlx::query_as(
            "SELECT dev.id, dev.collection_id, dev.name, dev.description, dev.mac, dev.file_hash, dev.created_at, dev.updated_at
             FROM devices as dev
             WHERE dev.collection_id = $1",
        )
        .bind(collection_id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(devices)
    }

    pub fn set_name(name: String) {
        todo!();
    }

    pub fn set_description(description: String) {
        todo!();
    }

    pub fn unset_description() {
        todo!();
    }
}
