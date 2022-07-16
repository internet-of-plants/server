use crate::{
    extractor::{Device, User},
    DeviceId, DevicePanic, DevicePanicId, DevicePanicView, NewDevicePanic, Pool, Result,
};
use axum::extract::{Extension, Json, Query};
use axum::http::StatusCode;
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
    let device = crate::Device::find_by_id(&mut txn, request.device_id, &user).await?;
    let device_panic = DevicePanic::find_by_id(&mut txn, &device, request.panic_id).await?;
    device_panic.solve(&mut txn).await?;
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
    Query(request): Query<ListRequest>,
) -> Result<Json<Vec<DevicePanicView>>> {
    let mut txn = pool.begin().await?;
    let device = crate::Device::find_by_id(&mut txn, request.device_id, &user).await?;
    let panics =
        DevicePanicView::first_n_from_device(&mut txn, &device, request.limit as i32).await?;
    Ok(Json(panics))
}
