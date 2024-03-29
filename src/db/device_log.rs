use crate::{logger::*, DateTime, Device, Error, Result, Transaction};
use derive::id;
use serde::{Deserialize, Serialize};

#[id]
pub struct DeviceLogId;

pub type DeviceLogView = DeviceLog;

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceLog {
    id: DeviceLogId,
    log: String,
    created_at: DateTime,
}

impl DeviceLog {
    pub async fn new(txn: &mut Transaction<'_>, device: &Device, log: String) -> Result<Self> {
        info!("Log (device_id: {:?}): {}", device.id(), log);
        let (id, now): (DeviceLogId, DateTime) = sqlx::query_as(
            "INSERT INTO device_logs (device_id, log) VALUES ($1, $2) RETURNING id, created_at",
        )
        .bind(device.id())
        .bind(&log)
        .fetch_one(txn)
        .await?;
        Ok(Self {
            id,
            log,
            created_at: now,
        })
    }

    pub fn log(&self) -> &str {
        &self.log
    }

    pub async fn first_n_from_device(
        txn: &mut Transaction<'_>,
        device: &Device,
        limit: i32,
    ) -> Result<Vec<Self>> {
        if limit > 10000 {
            return Err(Error::AskedForTooMany);
        }

        let device_logs: Vec<DeviceLog> = sqlx::query_as(
            "SELECT device_logs.id, device_logs.log, device_logs.created_at
            FROM device_logs
            WHERE device_id = $1
            ORDER BY device_logs.created_at DESC
            LIMIT $2",
        )
        .bind(device.id())
        .bind(limit)
        .fetch_all(txn)
        .await?;
        Ok(device_logs.into_iter().rev().collect())
    }
}
