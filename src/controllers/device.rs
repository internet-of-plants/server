use crate::db::collection::CollectionId;
use crate::db::device::{DeviceView, DeviceId};
use crate::db::workspace::WorkspaceId;
use crate::prelude::*;
use controllers::Result;

pub async fn find(
    workspace_id: WorkspaceId,
    collection_id: CollectionId,
    device_id: DeviceId,
    pool: &'static Pool,
    auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    // TODO: check if device belongs to collection and collection belongs to workspace and user is
    // in workspace
    let device = DeviceView::find_by_id(&mut txn, &device_id).await?; // TODO: Make it a DeviceView
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&device))
}
