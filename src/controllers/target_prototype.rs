use crate::db::board::BoardId;
use crate::db::target_prototype::{TargetPrototype, TargetPrototypeId};
use crate::prelude::*;
use controllers::Result;
use serde::Serialize;

#[derive(Serialize)]
pub struct BoardView {
    id: BoardId,
    name: String,
}

#[derive(Serialize)]
pub struct TargetPrototypeView {
    pub id: TargetPrototypeId,
    pub arch: String,
    pub build_flags: String,
    pub platform: String,
    pub framework: Option<String>,
    pub platform_packages: String,
    pub extra_platformio_params: String,
    pub ldf_mode: Option<String>,
    pub boards: Vec<BoardView>,
}

impl TargetPrototypeView {
    pub async fn new(txn: &mut Transaction<'_>, prototype: TargetPrototype) -> Result<Self> {
        let boards = prototype.boards(&mut *txn).await?;
        Ok(Self {
            id: prototype.id,
            arch: prototype.arch,
            build_flags: prototype.build_flags,
            platform: prototype.platform,
            framework: prototype.framework,
            platform_packages: prototype.platform_packages,
            extra_platformio_params: prototype.extra_platformio_params,
            ldf_mode: prototype.ldf_mode,
            boards: boards
                .into_iter()
                .map(|b| BoardView {
                    id: b.id(),
                    name: b.board,
                })
                .collect(),
        })
    }
}

pub async fn index(pool: &'static Pool, _auth: Auth) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    let prototypes = TargetPrototype::list(&mut txn).await?;
    let mut views = Vec::with_capacity(prototypes.len());
    for prototype in prototypes {
        views.push(TargetPrototypeView::new(&mut txn, prototype).await?);
    }

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&views))
}

pub async fn find(id: TargetPrototypeId, pool: &'static Pool, _auth: Auth) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    let prototype = TargetPrototype::find_by_id(&mut txn, id).await?;
    let view = TargetPrototypeView::new(&mut txn, prototype).await?;

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&view))
}
