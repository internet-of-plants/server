use crate::{extractor::User, Collection, CollectionId, CollectionView, Pool, Result};
use axum::extract::{Extension, Json, Query};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FindRequest {
    collection_id: CollectionId,
}

pub async fn find(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Query(request): Query<FindRequest>,
) -> Result<Json<CollectionView>> {
    let mut txn = pool.begin().await?;
    let collection = Collection::find_by_id(&mut txn, request.collection_id, &user).await?;
    let collection = CollectionView::new(&mut txn, collection).await?;
    txn.commit().await?;
    Ok(Json(collection))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    pub collection_id: CollectionId,
    pub name: String,
}

pub async fn set_name(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetNameRequest>,
) -> Result<Json<()>> {
    let mut txn = pool.begin().await?;
    let mut collection = Collection::find_by_id(&mut txn, request.collection_id, &user).await?;
    collection.set_name(&mut txn, request.name).await?;

    txn.commit().await?;
    Ok(Json(()))
}
