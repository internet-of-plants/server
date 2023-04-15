use crate::{extractor::User, Collection, CollectionId, Pool, Result};
use axum::extract::{Extension, Json};
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    #[copy]
    collection_id: CollectionId,
    name: String,
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
