use std::time::Duration;
use crate::prelude::*;
use controllers::Result;

pub async fn new(pool: &'static Pool, user_id: i64) -> Result<impl Reply> {
    Ok(warp::reply::json(&Id { id: api::plant::new(pool, user_id).await? }))
}

pub async fn get(pool: &'static Pool, user_id: i64, Id { id }: Id) -> Result<impl Reply> {
    Ok(warp::reply::json(&api::plant::get(pool, user_id, id).await?))
}

pub async fn history(pool: &'static Pool, user_id: i64, req: RequestHistory) -> Result<impl Reply> {
    let history = api::plant::history(pool, user_id, req.id, Duration::from_secs(req.since_secs_ago)).await?;
    Ok(warp::reply::json(&history))
}

pub async fn index(pool: &'static Pool, user_id: i64) -> Result<impl Reply> {
    Ok(warp::reply::json(&api::plant::index(pool, user_id).await?))
}
