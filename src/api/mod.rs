pub mod device_log;
pub mod device_panic;
pub mod event;
pub mod plant;
pub mod update;
pub mod user;

use crate::prelude::*;
use codegen::{cache, exec_time};

pub const OFFLINE_TIMEOUT: i64 = 270;

#[exec_time]
#[cache(valid_for = 30)]
pub async fn now(pool: &'static Pool) -> Result<u64> {
    loop {
        let now = sqlx::query_as::<_, (i64,)>("SELECT CAST(EXTRACT(EPOCH FROM NOW()) AS BIGINT)")
            .fetch_optional(pool)
            .await?
            .map_or(0, |(now,)| now);
        if now == 0 {
            error!(target: "now", "now is 0 bro");
            continue;
        }
        return Ok(now as u64);
    }
}
