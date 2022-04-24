use crate::prelude::*;
use crate::{CollectionId, DeviceId, DevicePanic, DevicePanicId, OrganizationId};
use crate::db::device_panic::NewDevicePanic;
use controllers::Result;

pub async fn solve(id: DevicePanicId, pool: &'static Pool, _auth: Auth) -> Result<impl Reply> {
    // TODO: enforce ownerships
    let mut txn = pool.begin().await.map_err(Error::from)?;
    DevicePanic::solve(&mut txn, id).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(StatusCode::OK)
}

pub async fn new(pool: &'static Pool, auth: Auth, mut error: NewDevicePanic) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    if error.msg.trim() != error.msg {
        error.msg = error.msg.trim().to_owned();
    }
    if let Some(device_id) = auth.device_id {
        DevicePanic::new(&mut txn, &device_id, error).await?;
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
    let panics = DevicePanic::first_n_from_device(&mut txn, &device_id, limit).await?;
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&panics))
}

//pub async fn index(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
//    Ok(warp::reply::json(
//        &db::device_panic::index(pool, auth.user_id, None).await?,
//    ))
//}
