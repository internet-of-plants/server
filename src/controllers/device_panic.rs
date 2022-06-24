use crate::db::device_panic::NewDevicePanic;
use crate::extractor::{Device, User};
use crate::prelude::*;
use crate::{DeviceId, DevicePanic, DevicePanicId};
use axum::extract::{Extension, Json};
use axum::http::StatusCode;
use controllers::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SolveRequest {
    device_id: DeviceId,
    panic_id: DevicePanicId,
}

pub async fn solve(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SolveRequest>,
) -> Result<StatusCode> {
    let mut txn = pool.begin().await?;
    let device = db::device::Device::find_by_id(&mut txn, request.device_id, &user).await?;
    DevicePanic::solve(&mut txn, &device, request.panic_id).await?;
    txn.commit().await?;
    Ok(StatusCode::OK)
}

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Device(device): Device,
    Json(mut error): Json<NewDevicePanic>,
) -> Result<StatusCode> {
    let mut txn = pool.begin().await?;

    if error.msg.trim() != error.msg {
        error.msg = error.msg.trim().to_owned();
    }
    DevicePanic::new(&mut txn, &device, error).await?;

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
    Json(request): Json<ListRequest>,
) -> Result<Json<Vec<DevicePanic>>> {
    let mut txn = pool.begin().await?;
    let device = db::device::Device::find_by_id(&mut txn, request.device_id, &user).await?;
    let panics = DevicePanic::first_n_from_device(&mut txn, &device, request.limit as i32).await?;
    Ok(Json(panics))
}
