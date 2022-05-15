use crate::db::device_panic::NewDevicePanic;
use crate::extractor::Authorization;
use crate::prelude::*;
use crate::{CollectionId, DeviceId, DevicePanic, DevicePanicId, OrganizationId};
use axum::extract::{Extension, Json, Path};
use axum::http::StatusCode;
use controllers::Result;

pub async fn solve(
    Path(id): Path<DevicePanicId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<StatusCode> {
    // TODO: enforce ownerships
    let mut txn = pool.begin().await?;
    DevicePanic::solve(&mut txn, id).await?;
    txn.commit().await?;
    Ok(StatusCode::OK)
}

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    Json(mut error): Json<NewDevicePanic>,
) -> Result<StatusCode> {
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
    Path(_organization_id): Path<OrganizationId>,
    Path(_collection_id): Path<CollectionId>,
    Path(device_id): Path<DeviceId>,
    Path(limit): Path<u32>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<Vec<DevicePanic>>> {
    // TODO: enforce ownerships
    let mut txn = pool.begin().await?;
    let panics = DevicePanic::first_n_from_device(&mut txn, &device_id, limit).await?;
    Ok(Json(panics))
}

//pub async fn index(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
//    Ok(warp::reply::json(
//        &db::device_panic::index(pool, auth.user_id, None).await?,
//    ))
//}
