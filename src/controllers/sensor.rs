use crate::{extractor::User, Collection, CollectionId, Error, Pool, Result, Sensor, SensorId};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetAliasRequest {
    pub collection_id: CollectionId,
    pub sensor_id: SensorId,
    pub alias: String,
}

pub async fn set_alias(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetAliasRequest>,
) -> Result<Json<()>> {
    let mut txn = pool.begin().await?;

    let collection = Collection::find_by_id(&mut txn, request.collection_id, &user).await?;

    if let Some(mut compiler) = collection.compiler(&mut txn).await? {
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
    pub collection_id: CollectionId,
    pub sensor_id: SensorId,
    pub color: String,
}

pub async fn set_color(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetColorRequest>,
) -> Result<Json<()>> {
    let mut txn = pool.begin().await?;

    let collection = Collection::find_by_id(&mut txn, request.collection_id, &user).await?;

    if let Some(mut compiler) = collection.compiler(&mut txn).await? {
        let sensor = Sensor::find_by_id(&mut txn, &compiler, request.sensor_id).await?;
        compiler.set_color(&mut txn, &sensor, request.color).await?;
    } else {
        return Err(Error::Unauthorized);
    }

    txn.commit().await?;
    Ok(Json(()))
}
