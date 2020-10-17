use crate::prelude::*;
use controllers::Result;
use codegen::exec_time;

#[exec_time]
pub async fn new(pool: &'static Pool, user_id: i64, event: NewEvent) -> Result<impl Reply> {
    info!(target: "event", "User: {}, Plant: {}", user_id, event.plant_id);
    api::event::new(pool, user_id, event).await?;
    Ok(StatusCode::OK)
}
