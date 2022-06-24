use crate::extractor::User;
use crate::{prelude::*, DeviceView};
use crate::{Collection, CollectionId, CollectionView, Device};
use axum::extract::{Extension, Json};
use controllers::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FindRequest {
    collection_id: CollectionId,
}

pub async fn find(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<FindRequest>,
) -> Result<Json<CollectionView>> {
    let mut txn = pool.begin().await?;
    let collection = Collection::find_by_id(&mut txn, request.collection_id, &user).await?;
    let devices = Device::from_collection(&mut txn, request.collection_id, &user).await?;
    let mut device_views = Vec::with_capacity(devices.len());
    for device in devices {
        device_views.push(DeviceView::new(&mut txn, device).await?);
    }
    let collection = CollectionView::new_from_collection_and_devices(collection, device_views)?;
    txn.commit().await?;
    Ok(Json(collection))
}
