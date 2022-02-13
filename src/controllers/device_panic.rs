use crate::{CollectionId, DeviceId, DevicePanic, DevicePanicId, WorkspaceId};
use crate::prelude::*;
use controllers::Result;

pub async fn solve(id: DevicePanicId, pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
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

pub async fn plant(pool: &'static Pool, auth: Auth, Id { id }: Id) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    //let panics = db::device_panic::plant(&mut txn, auth.user_id, id).await?;
    let panics: Vec<String> = vec![];
    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&panics))
}

pub async fn index(
    _workspace_id: WorkspaceId,
    _collection_id: CollectionId,
    device_id: DeviceId,
    limit: u32,
    pool: &'static Pool,
    auth: Auth,
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
