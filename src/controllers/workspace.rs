use crate::db::workspace::{Workspace, WorkspaceId, WorkspaceView};
use crate::prelude::*;
use controllers::Result;
use std::time::Duration;

pub async fn find(
    workspace_id: WorkspaceId,
    pool: &'static Pool,
    auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let workspace = WorkspaceView::find_by_id(&mut txn, &workspace_id).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&workspace))
}

pub async fn from_user(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let workspaces = Workspace::from_user(&mut txn, &auth.user_id).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&workspaces))
}
