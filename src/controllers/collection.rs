use crate::prelude::*;
use crate::{Collection, CollectionId, CollectionView, Device, OrganizationId};
use controllers::Result;
use crate::extractor::Authorization;
use axum::extract::{Extension, Path, Json};

pub async fn find(
    Path(_organization_id): Path<OrganizationId>,
    Path(collection_id): Path<CollectionId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<CollectionView>> {
    let mut txn = pool.begin().await?;
    // TODO: check that collection belongs to organization and user belongs to organization
    let collection = Collection::find_by_id(&mut txn, &collection_id).await?;
    let devices = Device::from_collection(&mut txn, &collection_id).await?;
    let collection = CollectionView::new_from_collection_and_devices(collection, devices)?;
    txn.commit().await?;
    Ok(Json(collection))
}
