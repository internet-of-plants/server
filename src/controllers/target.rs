use crate::{extractor::User, Pool, Result, Target, TargetPrototypeId, TargetView};
use axum::extract::{Extension, Json, Query};
use derive_get::Getters;
use serde::Deserialize;

#[derive(Getters, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    #[copy]
    prototype_id: TargetPrototypeId,
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    User(_user): User,
    Query(request): Query<ListRequest>,
) -> Result<Json<Vec<TargetView>>> {
    let mut txn = pool.begin().await?;
    let targets = Target::list_for_prototype(&mut txn, request.prototype_id).await?;
    let mut views = Vec::with_capacity(targets.len());
    for target in targets {
        views.push(TargetView::new(&mut txn, target).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}
