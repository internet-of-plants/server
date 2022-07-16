use crate::{
    extractor::User, Pool, Result, SensorPrototype, SensorPrototypeView, Target, TargetId,
};
use axum::extract::{Extension, Json, Query};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListForTargetRequest {
    target_id: TargetId,
}

pub async fn list_for_target(
    Extension(pool): Extension<&'static Pool>,
    User(_user): User,
    Query(request): Query<ListForTargetRequest>,
) -> Result<Json<Vec<SensorPrototypeView>>> {
    let mut txn = pool.begin().await?;

    let target = Target::find_by_id(&mut txn, request.target_id).await?;
    let prototypes = SensorPrototype::list(&mut txn).await?;
    let mut views = Vec::with_capacity(prototypes.len());
    for prototype in prototypes {
        views.push(SensorPrototypeView::new(&mut txn, prototype, &[&target]).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}
