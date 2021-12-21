use crate::prelude::*;
use codegen::exec_time;
use controllers::Result;

#[exec_time]
pub async fn new(pool: &'static Pool, auth: Auth, event: NewEvent, headers: warp::http::HeaderMap) -> Result<impl Reply> {
    debug!("{:?}", headers);
    let mac = headers.get("MAC_ADDRESS").ok_or(Error::NothingFound)?.to_str().map_err(|_|Error::BadData)?.to_string();
    let stat = DeviceStat {
        version: headers.get("VERSION").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.to_uppercase(),
        time_running: headers.get("TIME_RUNNING").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?,
        vcc: headers.get("VCC").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?,
        free_heap: headers.get("FREE_HEAP").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?,
        free_stack: headers.get("FREE_STACK").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?,
        biggest_free_heap_block: headers.get("BIGGEST_FREE_HEAP_BLOCK").ok_or(Error::BadData)?.to_str().map_err(|_|Error::BadData)?.parse().map_err(|_|Error::BadData)?,
    };

    if let Some(plant_id) = auth.plant_id {
        info!(target: "event", "User: {}, Device: {}, PlantId: {}", auth.user_id, mac, plant_id);
        let ret = api::event::new(pool, auth.user_id, event, plant_id, stat.version.clone()).await;

        if let Some(update) = api::update::get(pool, auth.user_id, plant_id).await? {
            if update.file_hash != stat.version {
                return Ok(http::Response::builder().header("LATEST_VERSION", update.file_hash).body(""))
            }
        }
        Ok(ret.and(Ok(http::Response::builder().body("")))?)
    } else {
        warn!(target: "event", "Not Found => User: {}, Device: {}", auth.user_id, mac);
        Err(Error::Forbidden)?
    }
}
