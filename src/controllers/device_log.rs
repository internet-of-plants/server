use crate::extractor::Authorization;
use crate::prelude::*;
use crate::{CollectionId, DeviceId, DeviceLog, OrganizationId};
use axum::extract::{Extension, Json, Path, RawBody};
use axum::http::StatusCode;
use controllers::Result;
use futures::StreamExt;

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    RawBody(mut log): RawBody,
) -> Result<StatusCode> {
    let mut txn = pool.begin().await?;
    let mut log_buffer = Vec::new();
    for log in log.next().await {
        log_buffer.extend(&log?);
    }
    let log = String::from_utf8_lossy(log_buffer.as_ref())
        .trim()
        .to_owned();
    if let Some(device_id) = auth.device_id {
        DeviceLog::new(&mut txn, &device_id, log).await?;
    } else {
        return Err(Error::Forbidden)?;
    }
    txn.commit().await?;
    Ok(StatusCode::OK)
}

pub async fn index(
    Path(_organization_id): Path<OrganizationId>,
    Path(_collection_id): Path<CollectionId>,
    Path(device_id): Path<DeviceId>,
    Path(limit): Path<u32>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<Vec<DeviceLog>>> {
    // TODO: enforce ownerships
    let mut txn = pool.begin().await?;
    let logs = DeviceLog::first_n_from_device(&mut txn, &device_id, limit).await?;
    txn.commit().await?;
    Ok(Json(logs))
}
