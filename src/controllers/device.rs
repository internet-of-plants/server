use crate::prelude::*;
use crate::{CollectionId, DeviceId, DeviceView, OrganizationId};
use controllers::Result;

pub async fn find(
    _organization_id: OrganizationId,
    _collection_id: CollectionId,
    device_id: DeviceId,
    pool: &'static Pool,
    _auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let device = DeviceView::find_by_id(&mut txn, /*auth.user_id,*/ &device_id).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&device))
}


//#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
//pub struct RequestHistory {
//    pub id: i64,
//    pub since_secs_ago: u64,
//}
//pub async fn history(pool: &'static Pool, auth: Auth, req: RequestHistory) -> Result<impl Reply> {
//    let mut txn = pool.begin().await.map_err(Error::from)?;
//    // TODO: easy DOS channel
//    let history = db::plant::history(
//        &mut txn,
//        auth.user_id,
//        DeviceId::new(req.id),
//        Duration::from_secs(req.since_secs_ago),
//    )
//    .await?;
//    txn.commit().await.map_err(Error::from)?;
//    Ok(warp::reply::json(&history))
//}
