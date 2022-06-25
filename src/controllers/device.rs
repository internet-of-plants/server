use crate::extractor::User;
use crate::{prelude::*, Device};
use crate::{DeviceId, DeviceView};
use axum::extract::{Extension, Json, Query};
use controllers::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FindRequest {
    device_id: DeviceId,
}

pub async fn find(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Query(request): Query<FindRequest>,
) -> Result<Json<DeviceView>> {
    let mut txn = pool.begin().await?;
    let device = Device::find_by_id(&mut txn, request.device_id, &user).await?;
    let device = DeviceView::new(&mut txn, device).await?;
    txn.commit().await?;
    Ok(Json(device))
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    device_id: DeviceId,
    name: String,
}

pub async fn set_name(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetNameRequest>,
) -> Result<Json<()>> {
    let mut txn = pool.begin().await?;
    let mut device = Device::find_by_id(&mut txn, request.device_id, &user).await?;
    device.set_name(&mut txn, request.name).await?;
    txn.commit().await?;
    Ok(Json(()))
}
