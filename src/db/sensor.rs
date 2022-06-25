pub mod config;
pub mod config_request;
pub mod config_type;
pub mod measurement;

use std::collections::HashSet;

use crate::db::sensor::config::Config;
use crate::db::sensor_prototype::*;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

use self::config::NewConfig;
use self::config_request::ConfigRequest;
use self::{config::ConfigView, measurement::Measurement};

use super::compiler::Compiler;
use super::device::Device;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSensor {
    pub prototype_id: SensorPrototypeId,
    pub alias: String,
    pub configs: Vec<NewConfig>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SensorView {
    pub id: SensorId,
    pub name: String,
    pub alias: String,
    pub dependencies: Vec<String>,
    pub includes: Vec<String>,
    pub definitions: Vec<String>,
    pub setups: Vec<String>,
    pub measurements: Vec<Measurement>,
    pub configurations: Vec<ConfigView>,
    pub prototype: SensorPrototypeView,
}

impl SensorView {
    pub async fn list_for_compiler(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
        device: &Device,
    ) -> Result<Vec<Self>> {
        let sensors_metadata: Vec<(SensorId, SensorPrototypeId, String)> = sqlx::query_as(
            "SELECT sensors.id, sensors.prototype_id, bt.alias
             FROM sensors
             INNER JOIN sensor_prototypes ON sensor_prototypes.id = sensors.prototype_id
             INNER JOIN sensor_belongs_to_compiler bt ON bt.sensor_id = sensors.id
             WHERE bt.compiler_id = $1 AND bt.device_id = $2",
        )
        .bind(compiler.id())
        .bind(device.id())
        .fetch_all(&mut *txn)
        .await?;

        let target = compiler.target(txn).await?;

        let mut sensors = Vec::with_capacity(sensors_metadata.len());
        for (id, prototype_id, alias) in sensors_metadata {
            let sensor = Sensor { id, prototype_id };
            let prototype = sensor.prototype(txn).await?;
            let sensor_configs = sensor.configs(txn).await?;

            let mut configurations = Vec::with_capacity(sensor_configs.len());
            for config in sensor_configs {
                configurations.push(ConfigView::new(txn, config).await?);
            }

            sensors.push(Self {
                id: sensor.id(),
                name: prototype.name().to_owned(),
                alias,
                dependencies: prototype.dependencies(txn).await?,
                includes: prototype.includes(txn).await?,
                definitions: prototype.definitions(txn).await?,
                setups: prototype.setups(txn).await?,
                measurements: prototype.measurements(txn).await?,
                configurations,
                prototype: SensorPrototypeView::new(txn, prototype, &[&target]).await?,
            });
        }

        Ok(sensors)
    }
}

pub type Dependency = String;
pub type Include = String;
pub type Definition = String;
pub type Setup = String;

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct SensorId(pub i64);

impl SensorId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Sensor {
    pub id: SensorId,
    pub prototype_id: SensorPrototypeId,
}

impl Sensor {
    pub async fn new(txn: &mut Transaction<'_>, mut new_sensor: NewSensor) -> Result<Self> {
        let mut uniq = HashSet::new();
        new_sensor
            .configs
            .iter()
            .map(|c| c.request_id)
            .all(|x| uniq.insert(x));
        if uniq.len() != new_sensor.configs.len() {
            // Duplicated configs
            todo!();
        }

        new_sensor
            .configs
            .sort_by(|a, b| a.request_id.cmp(&b.request_id));
        let serialized = new_sensor
            .configs
            .iter()
            .map(|c| format!("{}-{}", c.request_id.0, c.value))
            .collect::<Vec<_>>()
            .join(",");

        // TODO: think about using json instead of serialized string (injection/hijacking risks?)
        // Racy?
        let id: Option<(SensorId,)> = sqlx::query_as(
            "
            SELECT sensor_id
            FROM (SELECT sensor_id, string_agg(concat, ',') as str, COUNT(*) as count
                  FROM (SELECT concat(request_id, '-', value) as concat, sensor_id
                        FROM configs) as conf
                  GROUP BY sensor_id) as sub
            INNER JOIN sensors ON sensors.id = sensor_id
            WHERE prototype_id = $1
                  AND count = $2
                  AND str = $3",
        )
        .bind(&new_sensor.prototype_id)
        .bind(new_sensor.configs.len() as i64)
        .bind(&serialized)
        .fetch_optional(&mut *txn)
        .await?;

        let sensor = if let Some((id,)) = id {
            Self {
                id,
                prototype_id: new_sensor.prototype_id,
            }
        } else {
            let (id,): (SensorId,) =
                sqlx::query_as("INSERT INTO sensors (prototype_id) VALUES ($1) RETURNING id")
                    .bind(&new_sensor.prototype_id)
                    .fetch_one(&mut *txn)
                    .await?;

            let sensor = Self {
                id,
                prototype_id: new_sensor.prototype_id,
            };
            for config in new_sensor.configs {
                let request = ConfigRequest::find_by_id(&mut *txn, config.request_id).await?;
                Config::new(&mut *txn, &sensor, &request, config.value).await?;
            }
            sensor
        };
        Ok(sensor)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, sensor_id: SensorId) -> Result<Self> {
        let sensor = sqlx::query_as("SELECT id, prototype_id FROM sensors WHERE id = $1")
            .bind(&sensor_id)
            .fetch_one(&mut *txn)
            .await?;
        Ok(sensor)
    }

    pub fn id(&self) -> SensorId {
        self.id
    }

    pub async fn prototype(&self, txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
        SensorPrototype::find_by_id(txn, self.prototype_id).await
    }

    pub async fn configs(&self, txn: &mut Transaction<'_>) -> Result<Vec<Config>> {
        Config::find_by_sensor(txn, self.id).await
    }
}
