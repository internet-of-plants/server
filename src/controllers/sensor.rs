use crate::controllers::Result;
use crate::db::device::DeviceId;
use crate::db::sensor::{Sensor, SensorId};
use crate::extractor::User;
use crate::{prelude::*, Device};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetAliasRequest {
    device_id: DeviceId,
    sensor_id: SensorId,
    alias: String,
}

pub async fn set_alias(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetAliasRequest>,
) -> Result<Json<()>> {
    let mut txn = pool.begin().await?;
    let device = Device::find_by_id(&mut txn, request.device_id, &user).await?;
    if let Some(compiler) = device.compiler(&mut txn).await? {
        let sensor = Sensor::find_by_id(&mut txn, request.sensor_id).await?;
        device
            .set_alias(&mut txn, &compiler, &sensor, request.alias)
            .await?;
    }

    txn.commit().await?;
    Ok(Json(()))
}
