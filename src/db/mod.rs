pub mod collection;
pub mod device;
pub mod device_log;
pub mod device_panic;
pub mod event;
pub mod organization;
pub mod timestamp;
pub mod update;
pub mod user;

pub mod code_generation;
pub mod firmware;
pub mod sensor;
pub mod sensor_prototype;
pub mod target;
pub mod target_prototype;

use crate::prelude::*;

pub const OFFLINE_TIMEOUT: i64 = 270;

//#[cache(valid_for = 30)]
pub async fn now(txn: &mut Transaction<'_>) -> Result<u64> {
    loop {
        let now = sqlx::query_as::<_, (i64,)>("SELECT CAST(EXTRACT(EPOCH FROM NOW()) AS BIGINT)")
            .fetch_optional(&mut *txn)
            .await?
            .map_or(0, |(now,)| now);
        if now == 0 {
            error!(target: "now", "now is 0 bro");
            continue;
        }
        return Ok(now as u64);
    }
}
