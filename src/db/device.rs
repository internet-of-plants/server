use crate::controllers::firmware::FirmwareView;
use crate::db::timestamp::DateTime;
use crate::prelude::*;
use crate::utils::random_string;
use crate::{Collection, CollectionId, Event, Organization, UserId};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

use super::code_generation::{Compiler, CompilerId, CompilerView};
use super::event::EventView;
use super::firmware::{Firmware, FirmwareId};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct DeviceId(pub i64);

impl DeviceId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LastUpdate {
    version: String,
    file_hash: String,
    created_at: DateTime,
    updated_at: DateTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    pub id: DeviceId,
    pub name: String,
    pub description: Option<String>,
    pub mac: String,
    pub firmware: FirmwareView,
    pub compiler: Option<CompilerView>,
    pub last_update: Option<LastUpdate>,
    pub last_event: Option<EventView>,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

impl DeviceView {
    pub async fn find_by_id(txn: &mut Transaction<'_>, device_id: &DeviceId) -> Result<Self> {
        let device = Device::find_by_id(txn, device_id).await?;

        let firmware = device.current_firmware(txn).await?;
        let firmware = FirmwareView::new(txn, firmware).await?;

        let compiler = if let Some(id) = device.compiler_id {
            let compiler = Compiler::find_by_id(txn, id).await?;
            Some(CompilerView::new(txn, compiler).await?)
        } else {
            None
        };

        let last_event = device.last_event(txn).await?;
        let last_event = if let Some(last_event) = last_event {
            Some(EventView::new(txn, last_event).await?)
        } else {
            None
        };
        Ok(DeviceView {
            id: device.id,
            name: device.name,
            description: device.description,
            firmware,
            compiler,
            mac: device.mac,
            last_update: None, // TODO
            last_event,
            created_at: device.created_at,
            updated_at: device.updated_at,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Device {
    id: DeviceId,
    collection_id: CollectionId,
    firmware_id: FirmwareId,
    compiler_id: Option<CompilerId>,
    name: String,
    description: Option<String>,
    mac: String,
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

    pub async fn set_firmware_id(
        &mut self,
        txn: &mut Transaction<'_>,
        firmware_id: FirmwareId,
    ) -> Result<()> {
        self.firmware_id = firmware_id;
        sqlx::query("UPDATE devices SET firmware_id = $1 WHERE id = $2")
            .bind(firmware_id)
            .bind(self.id)
            .execute(txn)
            .await?;
        Ok(())
    }

    pub async fn set_compiler_id(
        &mut self,
        txn: &mut Transaction<'_>,
        compiler_id: Option<CompilerId>,
    ) -> Result<()> {
        self.compiler_id = compiler_id;
        sqlx::query("UPDATE devices SET compiler_id = $1 WHERE id = $2")
            .bind(compiler_id)
            .bind(self.id)
            .execute(txn)
            .await?;
        Ok(())
    }

    pub async fn put(
        txn: &mut Transaction<'_>,
        user_id: &UserId,
        new_device: NewDevice,
    ) -> Result<DeviceId> {
        let device = Self::try_find_by_mac(txn, &new_device.mac).await?;

        let firmware =
            if let Some(firmware) = Firmware::try_find_by_hash(txn, &new_device.file_hash).await? {
                firmware
            } else {
                Firmware::new_unknown(txn, new_device.file_hash.clone()).await?
            };

        let compiler_id = if let Some(compilation) = firmware.compilation(txn).await? {
            Some(compilation.compiler(txn).await?.id())
        } else {
            None
        };
        if let Some(mut device) = device {
            if device.firmware_id != firmware.id() {
                device.set_firmware_id(txn, firmware.id()).await?;
            }
            if let Some(compiler_id) = compiler_id {
                if device.compiler_id.is_none() {
                    device.set_compiler_id(txn, Some(compiler_id)).await?;
                }
            }
            return Ok(device.id);
        }

        let organization = Organization::default_for_user(txn, user_id).await?;
        let collection = Collection::new(txn, new_device.mac.clone(), organization.id()).await?;

        let name = format!("device-{}", random_string(5));
        let (id,) =
            sqlx::query_as::<_, (DeviceId,)>("INSERT INTO devices (mac, number_of_plants, collection_id, name, firmware_id, compiler_id) VALUES ($1, '1', $2, $3, $4, $5) RETURNING id")
                .bind(&new_device.mac)
                .bind(collection.id())
                .bind(name)
                .bind(firmware.id())
                .bind(compiler_id)
                .fetch_one(&mut *txn)
                .await?;
        Ok(id)
    }

    pub async fn try_find_by_mac(txn: &mut Transaction<'_>, mac: &str) -> Result<Option<Self>> {
        let device: Option<Self> = sqlx::query_as(
            "SELECT dev.id, dev.collection_id, dev.firmware_id, dev.compiler_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             WHERE dev.mac = $1",
        )
        .bind(mac)
        .fetch_optional(txn)
        .await?;
        Ok(device)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, device_id: &DeviceId) -> Result<Self> {
        let device: Self = sqlx::query_as(
            "SELECT dev.id, dev.collection_id, dev.firmware_id, dev.compiler_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             WHERE dev.id = $1",
        )
        .bind(device_id)
        .fetch_one(txn)
        .await?;
        Ok(device)
    }

    pub async fn from_collection(
        txn: &mut Transaction<'_>,
        collection_id: &CollectionId,
    ) -> Result<Vec<Self>> {
        let devices: Vec<Self> = sqlx::query_as(
            "SELECT dev.id, dev.collection_id, dev.firmware_id, dev.compiler_id, dev.name, dev.description, dev.mac, dev.created_at, dev.updated_at
             FROM devices as dev
             WHERE dev.collection_id = $1",
        )
        .bind(collection_id)
        .fetch_all(txn)
        .await?;
        Ok(devices)
    }

    pub async fn compiler(&self, txn: &mut Transaction<'_>) -> Result<Option<Compiler>> {
        if let Some(compiler_id) = self.compiler_id {
            Ok(Some(Compiler::find_by_id(txn, compiler_id).await?))
        } else {
            Ok(None)
        }
    }

    pub async fn current_firmware(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        Firmware::find_by_id(txn, self.firmware_id).await
    }

    pub async fn last_event(&self, txn: &mut Transaction<'_>) -> Result<Option<Event>> {
        Event::last_from_device(txn, &self.id).await
    }

    pub async fn update(
        txn: &mut Transaction<'_>,
        device_id: DeviceId,
    ) -> Result<Option<Firmware>> {
        let device = Device::find_by_id(txn, &device_id).await?;
        if let Some(compiler) = device.compiler(txn).await? {
            let compilation = compiler.latest_compilation(txn).await?;
            Ok(Some(compilation.firmware(txn).await?))
        } else {
            Ok(None)
        }
    }
}
