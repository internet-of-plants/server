use crate::{logger::*, DateTime, Device, Result, Transaction, Error};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewDevicePanic {
    pub file: String,
    pub line: i32,
    pub func: String,
    pub msg: String,
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct DevicePanicId(i64);

pub type DevicePanicView = DevicePanic;

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DevicePanic {
    id: DevicePanicId,
    file: String,
    line: i32,
    func: String,
    msg: String,
    created_at: DateTime,
}

impl DevicePanic {
    pub async fn new(
        txn: &mut Transaction<'_>,
        device: &Device,
        new_device_panic: NewDevicePanic,
    ) -> Result<Self> {
        info!("Log (device_id: {:?}): {:?}", device.id(), new_device_panic);
        let (id, now): (DevicePanicId, DateTime) = sqlx::query_as("INSERT INTO device_panics (device_id, \"file\", line, func, msg) VALUES ($1, $2, $3, $4, $5) RETURNING id, created_at")
            .bind(device.id())
            .bind(&new_device_panic.file)
            .bind(new_device_panic.line)
            .bind(&new_device_panic.func)
            .bind(&new_device_panic.msg)
            .fetch_one(txn)
            .await?;
        Ok(Self {
            id,
            file: new_device_panic.file,
            line: new_device_panic.line,
            func: new_device_panic.func,
            msg: new_device_panic.msg,
            created_at: now,
        })
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        device: &Device,
        id: DevicePanicId,
    ) -> Result<Self> {
        let panic = sqlx::query_as(
            "SELECT p.id, p.file, p.line, p.func, p.msg, p.created_at
            FROM device_panics as p
            WHERE p.device_id = $1 AND p.id = $2",
        )
        .bind(device.id())
        .bind(id)
        .fetch_one(txn)
        .await?;
        Ok(panic)
    }

    pub async fn first_n_from_device(
        txn: &mut Transaction<'_>,
        device: &Device,
        limit: i32,
    ) -> Result<Vec<Self>> {
        if limit > 10000 {
            return Err(Error::AskedForTooMany);
        }

        let device_panics: Vec<Self> = sqlx::query_as(
            "SELECT p.id, p.file, p.line, p.func, p.msg, p.created_at
            FROM device_panics as p
            WHERE p.device_id = $1
            ORDER BY p.created_at ASC
            LIMIT $2",
        )
        .bind(device.id())
        .bind(&limit)
        .fetch_all(txn)
        .await?;
        Ok(device_panics.into_iter().rev().collect())
    }

    pub async fn solve(self, txn: &mut Transaction<'_>) -> Result<()> {
        sqlx::query("UPDATE device_panics SET is_solved = TRUE, updated_at = NOW() WHERE id = $1")
            .bind(self.id)
            .execute(txn)
            .await?;
        Ok(())
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
}
