use crate::{extractor::User, Device, DeviceId, DeviceView, Pool, Result};
use axum::extract::{Extension, Json, Query};
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FindRequest {
    #[copy]
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

#[derive(Getters, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    #[copy]
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
    let old_name = device.name().to_owned();
    device.set_name(&mut txn, request.name).await?;

    let mut collection = device.collection(&mut txn).await?;
    if Device::from_collection(&mut txn, &collection).await?.len() == 1 {
        if collection.name() == &old_name {
            collection
                .set_name(&mut txn, device.name().to_owned())
                .await?;
        }
    }

    txn.commit().await?;
    Ok(Json(()))
}
