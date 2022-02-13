use crate::prelude::*;
use crate::{Collection, CollectionId, CollectionView, Device, WorkspaceId};
use controllers::Result;

pub async fn find(
    _workspace_id: WorkspaceId,
    collection_id: CollectionId,
    pool: &'static Pool,
    _auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    // TODO: check that collection belongs to workspace and user belongs to workspace
    let collection = Collection::find_by_id(&mut txn, &collection_id).await?;
    let devices = Device::from_collection(&mut txn, &collection_id).await?;
    let collection = CollectionView::new_from_collection_and_devices(collection, devices)?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&collection))
}
