use crate::prelude::*;
use controllers::Result;
use std::time::Duration;

pub async fn get(pool: &'static Pool, auth: Auth, Id { id }: Id) -> Result<impl Reply> {
    Ok(warp::reply::json(
        &api::plant::get(pool, auth.user_id, id).await?,
    ))
}

pub async fn history(pool: &'static Pool, auth: Auth, req: RequestHistory) -> Result<impl Reply> {
    // TODO: easy DOS channel
    let history = api::plant::history(
        pool,
        auth.user_id,
        req.id,
        Duration::from_secs(req.since_secs_ago),
    )
    .await?;
    Ok(warp::reply::json(&history))
}

pub async fn index(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
    Ok(warp::reply::json(&api::plant::index(pool, auth.user_id).await?))
}
