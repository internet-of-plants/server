use crate::db::sensor_prototype::{SensorPrototype, SensorPrototypeView};
use crate::db::target::{TargetId, Target};
use crate::extractor::User;
use crate::prelude::*;
use axum::extract::{Extension, Json, Query};
use controllers::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListForTargetRequest {
    target_id: TargetId,
}

pub async fn list_for_target(
    Extension(pool): Extension<&'static Pool>,
    User(_user): User,
    Query(request): Query<ListForTargetRequest>
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
