use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use derive_more::FromStr;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::device::Device;
use super::firmware::Firmware;
use super::sensor::measurement::MeasurementView;

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct EventId(i64);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EventView {
    pub measurements: serde_json::Value,
    pub metadatas: Vec<MeasurementView>,
    pub created_at: DateTime,
}

impl EventView {
    pub fn new(event: Event) -> Result<Self> {
        Ok(Self {
            measurements: event.measurements,
            metadatas: serde_json::from_value(event.metadatas)?,
            created_at: event.created_at,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Event {
    pub id: EventId,
    pub measurements: serde_json::Value,
    pub metadatas: serde_json::Value,
    pub firmware_hash: String,
    pub created_at: DateTime,
}

impl Event {
    pub fn id(&self) -> &EventId {
        &self.id
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        device: &Device,
        measurements: serde_json::Value,
        firmware_hash: String,
    ) -> Result<Self> {
        let firmware = Firmware::try_find_by_hash(txn, &firmware_hash).await?;
        let compilation = if let Some(firmware) = firmware {
            firmware.compilation(txn).await?
        } else {
            None
        };
        let metadatas = if let Some(compilation) = compilation {
            let compiler = compilation.compiler(txn).await?;
            let sensors = compiler.sensors(txn, device).await?;
            let mut measurements = Vec::new();
            for (index, sensor) in sensors.into_iter().enumerate() {
                let prototype = &sensor.prototype;
                measurements.extend(
                    prototype
                        .measurements
                        .iter()
                        .map(|m| {
                            let reg = Handlebars::new();
                            let name = reg.render_template(&m.name, &json!({ "index": index }))?;
                            Ok(MeasurementView::new(m.clone(), name))
                        })
                        .collect::<Result<Vec<_>, Error>>()?,
                );
            }
            measurements
        } else {
            Vec::new()
        };
        let metadatas = serde_json::to_value(metadatas)?;

        //db::plant::owns(txn, user_id, farm_id).await?;
        let (id,): (EventId,) =
            sqlx::query_as("INSERT INTO events (device_id, measurements, metadatas, firmware_hash) VALUES ($1, $2, $3, $4) RETURNING id")
                .bind(device.id())
                .bind(&measurements)
                .bind(&metadatas)
                .bind(&firmware_hash)
                .fetch_one(&mut *txn)
                .await?;
        Ok(Self {
            id,
            measurements,
            metadatas,
            firmware_hash,
            created_at: now(), // TODO: fix this
        })
    }

    pub async fn last_from_device(
        txn: &mut Transaction<'_>,
        device: &Device,
    ) -> Result<Option<Self>> {
        let event: Option<Event> = sqlx::query_as(
            "SELECT id, measurements, metadatas, firmware_hash, created_at
            FROM events
            WHERE device_id = $1
            ORDER BY created_at DESC",
        )
        .bind(device.id())
        .fetch_optional(txn)
        .await?;
        debug!("Last Event: {:?}", event);
        Ok(event)
    }

    pub async fn list(txn: &mut Transaction<'_>, device: &Device, limit: u32) -> Result<Vec<Self>> {
        let event: Vec<Event> = sqlx::query_as(
            "SELECT id, measurements, metadatas, firmware_hash, created_at
            FROM events
            WHERE device_id = $1
            ORDER BY created_at DESC
            LIMIT $2",
        )
        .bind(device.id())
        .bind(limit as i64)
        .fetch_all(txn)
        .await?;
        Ok(event)
    }
}
