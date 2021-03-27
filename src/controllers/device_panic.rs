use crate::prelude::*;
use controllers::Result;

pub async fn solve(pool: &'static Pool, user_id: i64, Id { id }: Id) -> Result<impl Reply> {
    api::device_panic::solve(pool, user_id, id).await?;
    Ok(StatusCode::OK)
}

pub async fn new(pool: &'static Pool, user_id: i64, error: NewDevicePanic, mac: String) -> Result<impl Reply> {
    api::device_panic::new(pool, user_id, error, mac).await?;
    Ok(StatusCode::OK)
}

pub async fn index(pool: &'static Pool, user_id: i64) -> Result<impl Reply> {
    Ok(warp::reply::json(
        &api::device_panic::index(pool, user_id, None).await?,
    ))
}
