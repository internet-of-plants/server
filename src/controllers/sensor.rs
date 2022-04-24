use crate::controllers::Result;
use crate::db::sensor::config::Config;
use crate::db::sensor::{Measurement, NewSensor, Sensor, SensorId};
use crate::db::sensor_prototype::SensorPrototypeId;
use crate::prelude::*;
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
struct SensorView {
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

pub async fn list(pool: &'static Pool, auth: Auth) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let sensors = Sensor::list(&mut txn, auth.user_id).await?;
    let mut views = Vec::with_capacity(sensors.len());
    for sensor in sensors {
        views.push(SensorView::new(&mut txn, sensor).await?);
    }

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&views))
}

pub async fn list_for_prototype(
    sensor_prototype_id: SensorPrototypeId,
    pool: &'static Pool,
    auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;
    let sensors = Sensor::list_for_prototype(&mut txn, auth.user_id, sensor_prototype_id).await?;
    let mut views = Vec::with_capacity(sensors.len());
    for sensor in sensors {
        views.push(SensorView::new(&mut txn, sensor).await?);
    }

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&views))
}

pub async fn new(pool: &'static Pool, auth: Auth, new_sensor: NewSensor) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    let sensor = Sensor::new(&mut txn, auth.user_id, new_sensor).await?;
    let view = SensorView::new(&mut txn, sensor).await?;

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&view))
}
