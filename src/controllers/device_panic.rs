use crate::prelude::*;
use controllers::Result;

pub async fn solve(pool: &'static Pool, auth: Auth, Id { id }: Id) -> Result<impl Reply> {
    api::device_panic::solve(pool, auth.user_id, id).await?;
    Ok(StatusCode::OK)
}

pub async fn new(pool: &'static Pool, auth: Auth, mut error: NewDevicePanic) -> Result<impl Reply> {
    error.msg = error.msg.trim().to_owned();
    if let Some(plant_id) = auth.plant_id {
        api::device_panic::new(pool, auth.user_id, error, plant_id).await?;
    } else {
        return Err(Error::Forbidden)?;
    }
    Ok(StatusCode::OK)
}

pub async fn plant(pool: &'static Pool, auth: Auth, Id { id }: Id) -> Result<impl Reply> {
    Ok(warp::reply::json(
        &api::device_panic::index(pool, auth.user_id, Some(id)).await?,
    ))
}

pub async fn index(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
    Ok(warp::reply::json(
        &api::device_panic::index(pool, auth.user_id, None).await?,
    ))
}
