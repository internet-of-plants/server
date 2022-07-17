use crate::{extractor::User, Device, DeviceId, DeviceView, Pool, Result};
use axum::extract::{Extension, Json, Query};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    pub device_id: DeviceId,
    pub name: String,
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
