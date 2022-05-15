use crate::controllers::Result;
use crate::db::sensor::config::Config;
use crate::db::sensor::{Measurement, NewSensor, Sensor, SensorId};
use crate::db::sensor_prototype::SensorPrototypeId;
use crate::extractor::Authorization;
use crate::prelude::*;
use axum::extract::{Extension, Json, Path};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ConfigView {
    name: String,
    type_name: String,
    value: String,
}

impl ConfigView {
    pub async fn new(txn: &mut Transaction<'_>, config: Config) -> Result<Self> {
        let request = config.request(&mut *txn).await?;
        Ok(Self {
            type_name: request.ty(&mut *txn).await?.name,
            name: request.name,
            value: config.value,
        })
    }
}

#[derive(Serialize, Debug)]
pub struct SensorView {
    id: SensorId,
    name: String,
    dependencies: Vec<String>,
    includes: Vec<String>,
    definitions: Vec<String>,
    setups: Vec<String>,
    measurements: Vec<Measurement>,
    configurations: Vec<ConfigView>,
}

impl SensorView {
    pub async fn new(txn: &mut Transaction<'_>, sensor: Sensor) -> Result<Self> {
        let prototype = sensor.prototype(&mut *txn).await?;
        let sensor_configs = sensor.configs(&mut *txn).await?;
        let mut configurations = Vec::with_capacity(sensor_configs.len());
        for config in sensor_configs {
            configurations.push(ConfigView::new(&mut *txn, config).await?);
        }
        Ok(Self {
            id: sensor.id(),
            name: prototype.name().to_owned(),
            dependencies: prototype.dependencies(&mut *txn).await?,
            includes: prototype.includes(&mut *txn).await?,
            definitions: prototype.definitions(&mut *txn).await?,
            setups: prototype.setups(&mut *txn).await?,
            measurements: prototype.measurements(&mut *txn).await?,
            configurations,
        })
    }
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
) -> Result<Json<Vec<SensorView>>> {
    let mut txn = pool.begin().await?;
    let sensors = Sensor::list(&mut txn, auth.user_id).await?;
    let mut views = Vec::with_capacity(sensors.len());
    for sensor in sensors {
        views.push(SensorView::new(&mut txn, sensor).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}
pub async fn list_for_prototype(
    Path(sensor_prototype_id): Path<SensorPrototypeId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
) -> Result<Json<Vec<SensorView>>> {
    let mut txn = pool.begin().await?;
    let sensors = Sensor::list_for_prototype(&mut txn, auth.user_id, sensor_prototype_id).await?;
    let mut views = Vec::with_capacity(sensors.len());
    for sensor in sensors {
        views.push(SensorView::new(&mut txn, sensor).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    Json(new_sensor): Json<NewSensor>,
) -> Result<Json<SensorView>> {
    let mut txn = pool.begin().await?;

    let sensor = Sensor::new(&mut txn, auth.user_id, new_sensor).await?;
    let view = SensorView::new(&mut txn, sensor).await?;

    txn.commit().await?;
    Ok(Json(view))
}
