use crate::prelude::*;
use codegen::exec_time;
use controllers::Result;

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, event: NewEvent, mac: String) -> Result<impl Reply> {
    info!(target: "event", "User: {}, Device: {}", user_id, mac);
    api::event::new(pool, user_id, event, mac).await?;
    Ok(StatusCode::OK)
}
