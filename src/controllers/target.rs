use crate::db::board::BoardId;
use crate::db::target::{Target, TargetId};
use crate::db::target_prototype::TargetPrototypeId;
use crate::prelude::*;
use controllers::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTarget {
    target_prototype_id: TargetPrototypeId,
    board_id: BoardId,
}

#[derive(Serialize, Debug)]
struct TargetView {
    id: TargetId,
    arch: String,
    build_flags: String,
    platform: String,
    framework: Option<String>,
    platform_packages: String,
    extra_platformio_params: String,
    ldf_mode: Option<String>,
    board: String,
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

pub async fn list(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let targets = Target::list(&mut txn, auth.user_id).await?;
    let mut views = Vec::with_capacity(targets.len());
    for target in targets {
        views.push(TargetView::new(&mut txn, target).await?);
    }

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&views))
}

pub async fn list_for_prototype(
    target_prototype_id: TargetPrototypeId,
    pool: &'static Pool,
    auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let targets = Target::list_for_prototype(&mut txn, auth.user_id, target_prototype_id).await?;
    let mut views = Vec::with_capacity(targets.len());
    for target in targets {
        views.push(TargetView::new(&mut txn, target).await?);
    }

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&views))
}

pub async fn new(pool: &'static Pool, auth: Auth, new_target: NewTarget) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    let target = Target::new(
        &mut txn,
        new_target.board_id,
        auth.user_id,
        new_target.target_prototype_id,
    )
    .await?;
    let view = TargetView::new(&mut txn, target).await?;

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&view))
}
