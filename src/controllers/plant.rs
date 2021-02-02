use crate::prelude::*;
use controllers::Result;
use std::time::Duration;

pub async fn put(pool: &'static Pool, user_id: i64, mac: Mac) -> Result<impl Reply> {
    Ok(api::plant::put(pool, user_id, mac.mac).await?.to_string())
}

pub async fn get(pool: &'static Pool, user_id: i64, Id { id }: Id) -> Result<impl Reply> {
    Ok(warp::reply::json(
        &api::plant::get(pool, user_id, id).await?,
    ))
}

pub async fn history(pool: &'static Pool, user_id: i64, req: RequestHistory) -> Result<impl Reply> {
    // TODO: easy DOS channel
    let history = api::plant::history(
        pool,
        user_id,
        req.id,
        Duration::from_secs(req.since_secs_ago),
    )
    .await?;
    Ok(warp::reply::json(&history))
}

pub async fn index(pool: &'static Pool, user_id: i64) -> Result<impl Reply> {
    Ok(warp::reply::json(&api::plant::index(pool, user_id).await?))
}
