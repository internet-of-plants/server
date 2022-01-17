use crate::prelude::*;
use codegen::exec_time;
use controllers::Result;
use crate::db::event::{NewEvent, Event};

#[exec_time]
pub async fn new(
    pool: &'static Pool,
    auth: Auth,
    event: NewEvent,
    headers: warp::http::HeaderMap,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    let mac = headers
        .get("MAC_ADDRESS")
        .ok_or(Error::NothingFound)?
        .to_str()
        .map_err(|_| Error::BadData)?
        .to_string();
    let stat = DeviceStat {
        version: headers
            .get("VERSION")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .to_uppercase(),
        time_running: headers
            .get("TIME_RUNNING")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .parse()
            .map_err(|_| Error::BadData)?,
        vcc: headers
            .get("VCC")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .parse()
            .map_err(|_| Error::BadData)?,
        free_dram: headers
            .get("FREE_DRAM")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .parse()
            .map_err(|_| Error::BadData)?,
        free_iram: headers
            .get("FREE_IRAM")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .parse()
            .map_err(|_| Error::BadData)?,
        free_stack: headers
            .get("FREE_STACK")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .parse()
            .map_err(|_| Error::BadData)?,
        biggest_dram_block: headers
            .get("BIGGEST_DRAM_BLOCK")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .parse()
            .map_err(|_| Error::BadData)?,
        biggest_iram_block: headers
            .get("BIGGEST_IRAM_BLOCK")
            .ok_or(Error::BadData)?
            .to_str()
            .map_err(|_| Error::BadData)?
            .parse()
            .map_err(|_| Error::BadData)?,
    };

    if let Some(device_id) = auth.device_id {
        info!(target: "event", "User: {:?}, MAC: {}, DeviceId: {:?}, Stat: {:?}", auth.user_id, mac, device_id, stat);
        let event = Event::new(
            &mut txn,
            &device_id,
            event,
            stat.version,
        )
        .await;

        //if let Some(update) = db::update::get(&mut txn, auth.user_id, device_id).await? {
        //    if update.file_hash != stat.version {
        //        txn.commit().await.map_err(Error::from)?;
        //        return Ok(http::Response::builder()
        //            .header("LATEST_VERSION", update.file_hash)
        //            .body(""));
        //    }
        //}
        txn.commit().await.map_err(Error::from)?;
        Ok(http::Response::builder().body(""))
    } else {
        warn!(target: "event", "Not Found => User: {:?}, Device: {}", auth.user_id, mac);
        Err(Error::Forbidden)?
    }
}
