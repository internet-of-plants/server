use crate::db::target_prototype::{TargetPrototype, TargetPrototypeId};
use crate::extractor::Authorization;
use crate::prelude::*;
use axum::extract::{Extension, Json, Path};
use controllers::Result;

pub async fn index(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<Vec<TargetPrototype>>> {
    let mut txn = pool.begin().await?;

    let prototypes = TargetPrototype::list(&mut txn).await?;

    txn.commit().await?;
    Ok(Json(prototypes))
}

pub async fn find(
    Path(id): Path<TargetPrototypeId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<TargetPrototype>> {
    let mut txn = pool.begin().await?;

    let prototype = TargetPrototype::find_by_id(&mut txn, id).await?;

    txn.commit().await?;
    Ok(Json(prototype))
}
