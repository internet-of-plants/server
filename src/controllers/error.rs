use crate::prelude::*;
use controllers::Result;

pub async fn solve(pool: &'static Pool, user_id: i64, Id { id }: Id) -> Result<impl Reply> {
    api::error::solve(pool, user_id, id).await?;
    Ok(StatusCode::OK)
}

pub async fn new(pool: &'static Pool, user_id: i64, error: ErrorReport) -> Result<impl Reply> {
    api::error::new(pool, user_id, error).await?;
    Ok(StatusCode::OK)
}

pub async fn index(pool: &'static Pool, user_id: i64) -> Result<impl Reply> {
    Ok(warp::reply::json(&api::error::index(pool, user_id).await?))
}
