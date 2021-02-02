use crate::prelude::*;
use codegen::exec_time;
use controllers::Result;

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, event: NewEvent) -> Result<impl Reply> {
    info!(target: "event", "User: {}, Device: {}", user_id, event.mac);
    api::event::new(pool, user_id, event).await?;
    Ok(StatusCode::OK)
}
