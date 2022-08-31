use crate::{logger::*, DateTime, Device, Firmware, Result, SensorMeasurementView, Transaction};
use derive_more::FromStr;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceStat {
    pub version: String,
    pub time_running: u64,
    pub vcc: u16,
    pub free_dram: u64,
    pub free_iram: Option<u64>,
    pub free_stack: u32,
    pub biggest_dram_block: u64,
    pub biggest_iram_block: Option<u64>,
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct EventId(i64);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EventView {
    pub measurements: serde_json::Value,
    pub stat: DeviceStat,
    pub metadatas: Vec<SensorMeasurementView>,
    pub created_at: DateTime,
}

impl EventView {
    pub fn new(event: Event) -> Result<Self> {
        Ok(Self {
            measurements: event.measurements,
            metadatas: serde_json::from_value(event.metadatas)?,
            created_at: event.created_at,
            stat: serde_json::from_value(event.stat)?,
        })
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub id: EventId,
    pub measurements: serde_json::Value,
    pub stat: serde_json::Value,
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
        stat: DeviceStat,
    ) -> Result<Self> {
        let collection = device.collection(txn).await?;
        let organization = collection.organization(txn).await?;

        let firmware = Firmware::try_find_by_hash(txn, &organization, &stat.version).await?;
        let compilation = if let Some(firmware) = firmware {
            firmware.compilation(txn).await?
        } else {
            None
        };
        let metadatas = if let Some(compilation) = compilation {
            let compiler = compilation.compiler(txn).await?;
            let sensors = compiler.sensors(txn).await?;
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
                            Ok(SensorMeasurementView::new(
                                m.clone(),
                                name,
                                sensor.color.clone(),
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                );
            }
            measurements
        } else {
            Vec::new()
        };
        let metadatas = serde_json::to_value(metadatas)?;

        let stat_json = serde_json::to_value(&stat)?;
        let (id, now): (EventId, DateTime) =
            sqlx::query_as("INSERT INTO events (device_id, measurements, metadatas, firmware_hash, stat) VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at")
                .bind(device.id())
                .bind(&measurements)
                .bind(&metadatas)
                .bind(&stat.version)
                .bind(&stat_json)
                .fetch_one(&mut *txn)
                .await?;
        Ok(Self {
            id,
            measurements,
            metadatas,
            stat: stat_json,
            firmware_hash: stat.version,
            created_at: now,
        })
    }

    pub async fn last_from_device(
        txn: &mut Transaction<'_>,
        device: &Device,
    ) -> Result<Option<Self>> {
        let event: Option<Event> = sqlx::query_as(
            "SELECT id, measurements, metadatas, firmware_hash, stat, created_at
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

    pub async fn list(
        txn: &mut Transaction<'_>,
        device: &Device,
        since: DateTime,
    ) -> Result<Vec<Self>> {
        let event: Vec<Event> = sqlx::query_as(
            "SELECT id, measurements, metadatas, firmware_hash, stat, created_at
            FROM events
            WHERE device_id = $1 AND created_at >= $2
            ORDER BY created_at DESC",
        )
        .bind(device.id())
        .bind(since)
        .fetch_all(txn)
        .await?;
        Ok(event)
    }
}
