use crate::db::timestamp::{now, DateTime};
use crate::prelude::*;
use crate::DeviceId;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct NewDevicePanic {
    pub file: String,
    pub line: u32,
    pub func: String,
    pub msg: String,
}

//#[exec_time]
//#[cache(valid_for = 3600)]
//pub async fn owns(txn: &mut Transaction<'_>, user_id: i64, error_id: i64) -> Result<()> {
//    let exists: Option<(i32,)> = sqlx::query_as(
//        "SELECT 1
//        FROM device_panics
//        WHERE device_panics.owner_id = $1
//              AND device_panics.id = $2",
//    )
//    .bind(user_id)
//    .bind(error_id)
//    .fetch_optional(txn)
//    .await?;
//    match exists {
//        Some(_) => Ok(()),
//        None => Err(Error::NothingFound),
//    }
//}
//
#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct DevicePanicId(i64);

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DevicePanic {
    id: DevicePanicId,
    file: String,
    line: u32,
    func: String,
    msg: String,
    created_at: DateTime,
}

impl DevicePanic {
    pub async fn new(
        txn: &mut Transaction<'_>,
        device_id: &DeviceId,
        new_device_panic: NewDevicePanic,
    ) -> Result<Self> {
        // TODO: auditing event with history actor
        info!("Log (device_id: {:?}): {:?}", device_id, new_device_panic);
        let (id,): (DevicePanicId,) = sqlx::query_as("INSERT INTO device_panics (device_id, \"file\", line, func, msg) VALUES ($1, $2, $3, $4, $5) RETURNING id")
            .bind(device_id)
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
            created_at: now(),
        })
    }

    pub async fn first_n_from_device(
        txn: &mut Transaction<'_>,
        device_id: &DeviceId,
        limit: u32,
    ) -> Result<Vec<Self>> {
        let device_panics: Vec<DevicePanic> = sqlx::query_as(
            "SELECT p.id, p.file, p.line, p.func, p.msg, p.created_at
            FROM device_panics as p
            WHERE p.device_id = $1
            ORDER BY p.created_at DESC
            LIMIT $2",
        )
        .bind(device_id)
        .bind(&limit)
        .fetch_all(txn)
        .await?;
        Ok(device_panics.into_iter().rev().collect())
    }

    pub async fn solve(txn: &mut Transaction<'_>, device_panic_id: DevicePanicId) -> Result<()> {
        //db::device_panic::owns(txn, user_id, error_id).await?;
        sqlx::query("UPDATE device_panics SET is_solved = TRUE WHERE id = $1")
            .bind(&device_panic_id)
            .execute(txn)
            .await?;
        Ok(())
    }
}
