use crate::prelude::*;
use codegen::exec_time;
use controllers::Result;

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, event: NewEvent, headers: warp::http::HeaderMap) -> Result<impl Reply> {
    debug!("{:?}", headers);
    let mac = headers.get("MAC_ADDRESS").ok_or(Error::NothingFound)?.to_str().map_err(|_|Error::BadData)?.to_string();
    let version = headers.get("VERSION").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.to_string();
    let time_running: u16 = headers.get("TIME_RUNNING").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?;
    let vcc: u16 = headers.get("VCC").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?;
    let free_heap: u32 = headers.get("FREE_HEAP").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?;
    let free_stack: u32 = headers.get("FREE_STACK").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?;
    let biggest_free_heap_block: u32 = headers.get("BIGGEST_FREE_HEAP_BLOCK").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?;
    info!(target: "event", "User: {}, Device: {}", user_id, mac);
    api::event::new(pool, user_id, event, mac).await?;
    Ok(StatusCode::OK)
}
