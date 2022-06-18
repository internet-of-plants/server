use crate::controllers::Result;
use crate::db::sensor::config::Config;
use crate::db::sensor::config_request::ConfigRequestId;
use crate::db::sensor::{Measurement, Sensor, SensorId};
use crate::db::target::Target;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

use super::sensor_prototype::SensorPrototypeView;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConfigView {
    request_id: ConfigRequestId,
    name: String,
    type_name: String,
    value: String,
}

impl ConfigView {
    pub async fn new(txn: &mut Transaction<'_>, config: Config) -> Result<Self> {
        let request = config.request(&mut *txn).await?;
        Ok(Self {
            request_id: config.request_id,
            type_name: request.ty(&mut *txn).await?.name,
            name: request.name,
            value: config.value,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SensorView {
    pub id: SensorId,
    pub name: String,
    pub dependencies: Vec<String>,
    pub includes: Vec<String>,
    pub definitions: Vec<String>,
    pub setups: Vec<String>,
    pub measurements: Vec<Measurement>,
    pub configurations: Vec<ConfigView>,
    pub prototype: SensorPrototypeView
}

impl SensorView {
    pub async fn new(txn: &mut Transaction<'_>, sensor: Sensor, target: Target) -> Result<Self> {
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
            prototype: SensorPrototypeView::new(&mut *txn, prototype, &[target.id()]).await?,
        })
    }
}

//pub async fn list(
//    Extension(pool): Extension<&'static Pool>,
//    Authorization(_auth): Authorization,
//) -> Result<Json<Vec<SensorView>>> {
//    let mut txn = pool.begin().await?;
//    let sensors = Sensor::list(&mut txn).await?;
//    let mut views = Vec::with_capacity(sensors.len());
//    for sensor in sensors {
//        views.push(SensorView::new(&mut txn, sensor, target).await?);
//    }
//
//    txn.commit().await?;
//    Ok(Json(views))
//}

//pub async fn list_for_prototype(
//    Path(sensor_prototype_id): Path<SensorPrototypeId>,
//    Extension(pool): Extension<&'static Pool>,
//    Authorization(_auth): Authorization,
//) -> Result<Json<Vec<SensorView>>> {
//    let mut txn = pool.begin().await?;
//    let sensors = Sensor::list_for_prototype(&mut txn, sensor_prototype_id).await?;
//    let mut views = Vec::with_capacity(sensors.len());
//    for sensor in sensors {
//        views.push(SensorView::new(&mut txn, sensor).await?);
//    }
//
//    txn.commit().await?;
//    Ok(Json(views))
//}

//pub async fn new(
//    Extension(pool): Extension<&'static Pool>,
//    Authorization(_auth): Authorization,
//    Json(new_sensor): Json<NewSensor>,
//) -> Result<Json<SensorView>> {
//    let mut txn = pool.begin().await?;
//
//    let sensor = Sensor::new(&mut txn, new_sensor).await?;
//    let view = SensorView::new(&mut txn, sensor).await?;
//
//    txn.commit().await?;
//    Ok(Json(view))
//}
