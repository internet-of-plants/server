use crate::prelude::*;
use bytes::Bytes;
use controllers::Result;

pub async fn new(
    pool: &'static Pool,
    user_id: i64,
    log: Bytes,
    mac_address: String,
) -> Result<impl Reply> {
    let log = String::from_utf8_lossy(log.as_ref()).trim().to_owned();
    let plant_id = api::plant::put(pool, user_id, mac_address).await?;
    api::device_log::new(pool, user_id, plant_id, log).await?;
    Ok(StatusCode::OK)
}

pub async fn index(plant_id: String, pool: &'static Pool, user_id: i64) -> Result<impl Reply> {
    let plant_id = plant_id.parse::<i64>().map_err(|_| Error::NothingFound)?;
    Ok(warp::reply::json(
        &api::device_log::index(pool, user_id, plant_id).await?,
    ))
}
