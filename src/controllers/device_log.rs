use crate::prelude::*;
use bytes::Bytes;
use controllers::Result;

pub async fn new(
    pool: &'static Pool,
    auth: Auth,
    log: Bytes,
) -> Result<impl Reply> {
    let log = String::from_utf8_lossy(log.as_ref()).trim().to_owned();
    if let Some(plant_id) = auth.plant_id {
        api::device_log::new(pool, auth.user_id, plant_id, log).await?;
    } else {
        return Err(Error::Forbidden)?;
    }
    Ok(StatusCode::OK)
}

pub async fn index(plant_id: String, pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
    let plant_id = plant_id.parse::<i64>().map_err(|_| Error::NothingFound)?;
    Ok(warp::reply::json(
        &api::device_log::index(pool, auth.user_id, plant_id).await?,
    ))
}
