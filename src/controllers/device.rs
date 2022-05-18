use crate::extractor::Authorization;
use crate::prelude::*;
use crate::{CollectionId, DeviceId, DeviceView, OrganizationId};
use axum::extract::{Extension, Json, Path};
use controllers::Result;

type FindPath = (OrganizationId, CollectionId, DeviceId);
pub async fn find(
    Path((_organization_id, _collection_id, device_id)): Path<FindPath>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<DeviceView>> {
    let mut txn = pool.begin().await?;
    let device = DeviceView::find_by_id(&mut txn, /*auth.user_id,*/ &device_id).await?;
    txn.commit().await?;
    Ok(Json(device))
}

//#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
//pub struct RequestHistory {
//    pub id: i64,
//    pub since_secs_ago: u64,
//}
//pub async fn history(pool: &'static Pool, auth: Auth, req: RequestHistory) -> Result<impl Reply> {
//    let mut txn = pool.begin().await?;
//    // TODO: easy DOS channel
//    let history = db::plant::history(
//        &mut txn,
//        auth.user_id,
//        DeviceId::new(req.id),
//        Duration::from_secs(req.since_secs_ago),
//    )
//    .await?;
//    txn.commit().await?;
//    Ok(warp::reply::json(&history))
//}