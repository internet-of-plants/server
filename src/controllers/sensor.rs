use crate::{extractor::User, Device, DeviceId, Error, Pool, Result, Sensor, SensorId};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetAliasRequest {
    pub device_id: DeviceId,
    pub sensor_id: SensorId,
    pub alias: String,
}

pub async fn set_alias(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetAliasRequest>,
) -> Result<Json<()>> {
    let mut txn = pool.begin().await?;
    let device = Device::find_by_id(&mut txn, request.device_id, &user).await?;

    if let Some(mut compiler) = device.compiler(&mut txn).await? {
        let sensor = Sensor::find_by_id(&mut txn, &compiler, request.sensor_id).await?;
        compiler.set_alias(&mut txn, &sensor, request.alias).await?;
    } else {
        return Err(Error::Unauthorized);
    }

    txn.commit().await?;
    Ok(Json(()))
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetColorRequest {
    pub device_id: DeviceId,
    pub sensor_id: SensorId,
    pub color: String,
}

pub async fn set_color(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetColorRequest>,
) -> Result<Json<()>> {
    let mut txn = pool.begin().await?;

    let device = Device::find_by_id(&mut txn, request.device_id, &user).await?;

    if let Some(mut compiler) = device.compiler(&mut txn).await? {
        let sensor = Sensor::find_by_id(&mut txn, &compiler, request.sensor_id).await?;
        compiler.set_color(&mut txn, &sensor, request.color).await?;
    } else {
        return Err(Error::Unauthorized);
    }

    txn.commit().await?;
    Ok(Json(()))
}
