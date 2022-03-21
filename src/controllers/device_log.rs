use crate::prelude::*;
use crate::{CollectionId, DeviceId, DeviceLog, OrganizationId};
use bytes::Bytes;
use controllers::Result;

pub async fn new(pool: &'static Pool, auth: Auth, log: Bytes) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let log = String::from_utf8_lossy(log.as_ref()).trim().to_owned();
    if let Some(device_id) = auth.device_id {
        DeviceLog::new(&mut txn, &device_id, log).await?;
    } else {
        return Err(Error::Forbidden)?;
    }
    txn.commit().await.map_err(Error::from)?;
    Ok(StatusCode::OK)
}

pub async fn index(
    _organization_id: OrganizationId,
    _collection_id: CollectionId,
    device_id: DeviceId,
    limit: u32,
    pool: &'static Pool,
    _auth: Auth,
) -> Result<impl Reply> {
    // TODO: enforce ownerships
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let logs = DeviceLog::first_n_from_device(&mut txn, &device_id, limit).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&logs))
}
