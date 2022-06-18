use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::DeviceId;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

use super::firmware::Firmware;
use super::sensor::MeasurementType;

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct EventId(i64);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementView {
    name: String,
    ty: MeasurementType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventView {
    pub measurements: serde_json::Value,
    pub metadatas: Vec<MeasurementView>,
    pub created_at: DateTime,
}

impl EventView {
    pub async fn new(txn: &mut Transaction<'_>, event: Event) -> Result<Self> {
        // TODO: fix error types
        let firmware = Firmware::try_find_by_hash(txn, &event.firmware_hash).await?;
        let compilation = if let Some(firmware) = firmware {
            firmware.compilation(txn).await?
        } else {
            None
        };
        let metadata = if let Some(compilation) = compilation {
            let compiler = compilation.compiler(txn).await?;
            let sensors = compiler.sensors(txn).await?;
            let mut measurements = Vec::new();
            for sensor in sensors {
                let prototype = sensor.prototype(txn).await?;
                measurements.extend(prototype.measurements(txn).await?);
            }
            measurements
        } else {
            Vec::new()
        };
        Ok(Self {
            measurements: event.measurements,
            metadatas: metadata
                .into_iter()
                .map(|m| MeasurementView {
                    name: m.name,
                    ty: m.ty,
                })
                .collect(),
            created_at: event.created_at,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Event {
    pub id: EventId,
    pub measurements: serde_json::Value,
    pub firmware_hash: String,
    pub created_at: DateTime,
}

pub type NewEvent = serde_json::Value;

impl Event {
    pub fn id(&self) -> &EventId {
        &self.id
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        device_id: &DeviceId,
        measurements: NewEvent,
        firmware_hash: String,
    ) -> Result<Self> {
        //db::plant::owns(txn, user_id, farm_id).await?;
        // TODO: log error if something is NaN
        let (id,): (EventId,) =
            sqlx::query_as("INSERT INTO events (device_id, measurements, firmware_hash) VALUES ($1, $2, $3) RETURNING id")
                .bind(device_id)
                .bind(&measurements)
                .bind(&firmware_hash)
                .fetch_one(&mut *txn)
                .await?;
        Ok(Self {
            id,
            measurements,
            firmware_hash,
            created_at: now(), // TODO: fix this
        })
    }

    pub async fn last_from_device(
        txn: &mut Transaction<'_>,
        device_id: &DeviceId,
    ) -> Result<Option<Self>> {
        let event: Option<Event> = sqlx::query_as(
            "SELECT id, measurements, firmware_hash, created_at
            FROM events
            WHERE device_id = $1
            ORDER BY created_at DESC",
        )
        .bind(device_id)
        .fetch_optional(txn)
        .await?;
        debug!("Last Event: {:?}", event);
        Ok(event)
    }

    pub async fn list(
        txn: &mut Transaction<'_>,
        device_id: &DeviceId,
        limit: u32,
    ) -> Result<Vec<Self>> {
        let event: Vec<Event> = sqlx::query_as(
            "SELECT id, measurements, firmware_hash, created_at
            FROM events
            WHERE device_id = $1
            ORDER BY created_at DESC
            LIMIT $2",
        )
        .bind(device_id)
        .bind(limit as i64)
        .fetch_all(txn)
        .await?;
        Ok(event)
    }

    //pub async fn list_for_compiler(
    //    txn: &mut Transaction<'_>,
    //    device_id: &DeviceId,
    //    compiler_id: &CompilerId,
    //    limit: u32,
    //) -> Result<Vec<Self>> {
    //    let event: Vec<Event> = sqlx::query_as(
    //        "SELECT events.id, events.measurements, events.firmware_hash, events.created_at
    //        FROM events
    //        INNER JOIN devices ON devices.id = events.device_id
    //        WHERE events.device_id = $1
    //              devices.compiler_id = $2
    //        ORDER BY events.created_at DESC
    //        LIMIT $3",
    //    )
    //    .bind(device_id)
    //    .bind(compiler_id)
    //    .bind(limit as i64)
    //    .fetch_all(txn)
    //    .await?;
    //    Ok(event)
    //}
}
