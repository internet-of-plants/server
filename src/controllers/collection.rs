use crate::extractor::User;
use crate::prelude::*;
use crate::{Collection, CollectionId, CollectionView};
use axum::extract::{Extension, Json, Query};
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
    Query(request): Query<FindRequest>,
) -> Result<Json<CollectionView>> {
    let mut txn = pool.begin().await?;
    let collection = Collection::find_by_id(&mut txn, request.collection_id, &user).await?;
    let collection = CollectionView::new(&mut txn, collection, &user).await?;
    txn.commit().await?;
    Ok(Json(collection))
}
