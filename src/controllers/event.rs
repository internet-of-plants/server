use crate::extractor::Authorization;
use crate::extractor::{
    BiggestDramBlock, BiggestIramBlock, FreeDram, FreeIram, FreeStack, MacAddress, TimeRunning,
    Vcc, Version,
};
use crate::prelude::*;
use crate::{Event, NewEvent, Update};
use axum::extract::{Extension, Json, TypedHeader};
use axum::http::header::{HeaderMap, HeaderName, HeaderValue};
use controllers::Result;
use serde::Serialize;
use std::iter::FromIterator;

#[derive(Debug, Clone, PartialEq, Serialize)]
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

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    Json(event): Json<NewEvent>,
    TypedHeader(MacAddress(mac)): TypedHeader<MacAddress>,
    TypedHeader(Version(version)): TypedHeader<Version>,
    TypedHeader(TimeRunning(time_running)): TypedHeader<TimeRunning>,
    TypedHeader(Vcc(vcc)): TypedHeader<Vcc>,
    TypedHeader(FreeDram(free_dram)): TypedHeader<FreeDram>,
    TypedHeader(BiggestDramBlock(biggest_dram_block)): TypedHeader<BiggestDramBlock>,
    TypedHeader(FreeStack(free_stack)): TypedHeader<FreeStack>,
    free_iram: Option<TypedHeader<FreeIram>>,
    biggest_iram_block: Option<TypedHeader<BiggestIramBlock>>,
) -> Result<impl IntoResponse> {
    let stat = DeviceStat {
        version,
        time_running: time_running.parse()?,
        vcc: vcc.parse()?,
        free_dram: free_dram.parse()?,
        free_iram: free_iram.and_then(|TypedHeader(FreeIram(iram))| iram.parse().ok()),
        free_stack: free_stack.parse()?,
        biggest_dram_block: biggest_dram_block.parse()?,
        biggest_iram_block: biggest_iram_block
            .and_then(|TypedHeader(BiggestIramBlock(size))| size.parse().ok()),
    };

    if let Some(device_id) = auth.device_id {
        let mut txn = pool.begin().await?;

        info!(target: "event", "User: {:?}, MAC: {}, DeviceId: {:?}, Stat: {:?}", auth.user_id, mac, device_id, stat);
        Event::new(&mut txn, &device_id, event, stat.version.clone()).await?;

        if let Some(update) = Update::find_by_device(&mut txn, auth.user_id, device_id).await? {
            let firmware = update.firmware(&mut txn).await?;
            if firmware.hash() != &stat.version {
                txn.commit().await?;
                return Ok(HeaderMap::from_iter([(
                    HeaderName::from_static("LATEST_VERSION"),
                    HeaderValue::from_str(firmware.hash())?,
                )]));
            }
        }
        txn.commit().await?;
        Ok(HeaderMap::new())
    } else {
        warn!(target: "event", "Not Found => User: {:?}, Device: {}", auth.user_id, mac);
        Err(Error::Forbidden)?
    }
}
