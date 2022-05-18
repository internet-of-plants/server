use crate::db::board::BoardId;
use crate::db::target::{Target, TargetId};
use crate::db::target_prototype::TargetPrototypeId;
use crate::extractor::Authorization;
use crate::prelude::*;
use axum::extract::{Extension, Json, Path};
use controllers::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTarget {
    pub target_prototype_id: TargetPrototypeId,
    pub board_id: BoardId,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct TargetView {
    pub id: TargetId,
    pub arch: String,
    pub build_flags: String,
    pub platform: String,
    pub framework: Option<String>,
    pub platform_packages: String,
    pub extra_platformio_params: String,
    pub ldf_mode: Option<String>,
    pub board: String,
}

impl TargetView {
    pub async fn new(txn: &mut Transaction<'_>, target: Target) -> Result<Self> {
        let prototype = target.prototype(&mut *txn).await?;
        Ok(Self {
            id: target.id(),
            arch: prototype.arch,
            build_flags: prototype.build_flags,
            platform: prototype.platform,
            framework: prototype.framework,
            platform_packages: prototype.platform_packages,
            extra_platformio_params: prototype.extra_platformio_params,
            ldf_mode: prototype.ldf_mode,
            board: target.board(&mut *txn).await?.board,
        })
    }
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
) -> Result<Json<Vec<TargetView>>> {
    let mut txn = pool.begin().await?;
    let targets = Target::list(&mut txn, auth.user_id).await?;
    let mut views = Vec::with_capacity(targets.len());
    for target in targets {
        views.push(TargetView::new(&mut txn, target).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}

pub async fn list_for_prototype(
    Path(target_prototype_id): Path<TargetPrototypeId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
) -> Result<Json<Vec<TargetView>>> {
    let mut txn = pool.begin().await?;
    let targets = Target::list_for_prototype(&mut txn, auth.user_id, target_prototype_id).await?;
    let mut views = Vec::with_capacity(targets.len());
    for target in targets {
        views.push(TargetView::new(&mut txn, target).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    Json(new_target): Json<NewTarget>,
) -> Result<Json<TargetView>> {
    let mut txn = pool.begin().await?;

    let target = Target::new(
        &mut txn,
        new_target.board_id,
        auth.user_id,
        new_target.target_prototype_id,
    )
    .await?;
    let view = TargetView::new(&mut txn, target).await?;

    txn.commit().await?;
    Ok(Json(view))
}
