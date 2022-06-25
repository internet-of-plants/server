use crate::extractor::{Device, User};
use crate::prelude::*;
use crate::{DeviceId, DeviceLog};
use axum::extract::{Extension, Json, RawBody, Query};
use axum::http::StatusCode;
use controllers::Result;
use futures::StreamExt;
use serde::Deserialize;

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Device(device): Device,
    RawBody(mut log): RawBody,
) -> Result<StatusCode> {
    let mut txn = pool.begin().await?;

    let mut log_buffer = Vec::new();
    for log in log.next().await {
        log_buffer.extend(&log?);
    }
    let log = String::from_utf8_lossy(log_buffer.as_ref())
        .trim()
        .to_owned();

    DeviceLog::new(&mut txn, &device, log).await?;

    txn.commit().await?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    device_id: DeviceId,
    limit: u16,
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Query(request): Query<ListRequest>,
) -> Result<Json<Vec<DeviceLog>>> {
    let mut txn = pool.begin().await?;

    let device = db::device::Device::find_by_id(&mut txn, request.device_id, &user).await?;
    let logs = DeviceLog::first_n_from_device(&mut txn, &device, request.limit as i32).await?;

    txn.commit().await?;
    Ok(Json(logs))
}
