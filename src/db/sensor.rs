use crate::{
    db::sensor_config::Val, Compiler, Dependency, Error, NewSensorConfig, Result, SensorConfig,
    SensorConfigRequest, SensorConfigView, SensorMeasurementView, SensorPrototype,
    SensorPrototypeId, SensorPrototypeView, Transaction,
};
use derive::id;
use derive_get::Getters;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashSet, VecDeque};
use std::iter::FromIterator;

#[derive(Getters, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSensor {
    #[copy]
    prototype_id: SensorPrototypeId,
    alias: String,
    configs: Vec<NewSensorConfig>,
    // Slot identifier for sensor select
    #[copy]
    local_pk: usize,
}

impl NewSensor {
    pub fn configs_mut(&mut self) -> &mut [NewSensorConfig] {
        &mut self.configs
    }
}

#[derive(Getters, Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SensorView {
    #[copy]
    id: SensorId,
    name: String,
    alias: String,
    color: String,
    dependencies: Vec<Dependency>,
    includes: Vec<String>,
    definitions: Vec<Definition>,
    setups: Vec<String>,
    unauthenticated_actions: Vec<String>,
    measurements: Vec<SensorMeasurementView>,
    configurations: Vec<SensorConfigView>,
    prototype: SensorPrototypeView,
}

impl SensorView {
    pub async fn list_for_compiler(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
    ) -> Result<Vec<Self>> {
        let sensors_metadata: Vec<(SensorId, SensorPrototypeId, String, String)> = sqlx::query_as(
            "SELECT DISTINCT ON (sensors.id) sensors.id, sensors.prototype_id, bt.alias, bt.color
             FROM sensors
             INNER JOIN sensor_prototypes ON sensor_prototypes.id = sensors.prototype_id
             INNER JOIN sensor_belongs_to_compiler bt ON bt.sensor_id = sensors.id
             WHERE bt.compiler_id = $1
             ORDER BY sensors.id",
        )
        .bind(compiler.id())
        .fetch_all(&mut *txn)
        .await?;

        let target = compiler.target(txn).await?;

        let mut sensors = Vec::with_capacity(sensors_metadata.len());
        for (index, (id, prototype_id, alias, color)) in sensors_metadata.into_iter().enumerate() {
            let sensor = Sensor { id, prototype_id };
            let prototype = sensor.prototype(txn).await?;
            let sensor_configs = sensor.configs(txn).await?;

            let mut configurations = Vec::with_capacity(sensor_configs.len());
            for config in sensor_configs {
                configurations.push(SensorConfigView::new(txn, config).await?);
            }

            sensors.push(Self {
                id: sensor.id(),
                name: prototype.name().to_owned(),
                alias,
                color: color.clone(),
                dependencies: prototype.dependencies(txn).await?,
                includes: prototype.includes(txn).await?,
                definitions: prototype.definitions(txn).await?,
                setups: prototype.setups(txn).await?,
                unauthenticated_actions: prototype.unauthenticated_actions(txn).await?,
                measurements: prototype
                    .measurements(txn)
                    .await?
                    .iter()
                    .map(|m| {
                        let reg = Handlebars::new();
                        let name = reg.render_template(m.name(), &json!({ "index": index }))?;
                        Ok(SensorMeasurementView::new(m.clone(), name, color.clone()))
                    })
                    .collect::<Result<Vec<_>>>()?,
                configurations,
                prototype: SensorPrototypeView::new(txn, prototype, &[&target]).await?,
            });
        }

        Ok(sensors)
    }
}

pub type Include = String;

#[derive(sqlx::FromRow, Getters, Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct SensorReference {
    sensor_name: String,
    request_name: String,
}
impl SensorReference {
    pub fn new(sensor_name: String, request_name: String) -> Self {
        Self {
            sensor_name,
            request_name,
        }
    }
}

#[id]
pub struct SensorPrototypeDefinitionId;

impl SensorPrototypeDefinitionId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}
#[derive(sqlx::FromRow, Getters, Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Definition {
    line: String,
    sensors_referenced: Vec<SensorReference>,
}
impl Definition {
    pub fn new(line: String, sensors_referenced: Vec<SensorReference>) -> Self {
        Self {
            line,
            sensors_referenced,
        }
    }
}
impl From<String> for Definition {
    fn from(line: String) -> Self {
        Self {
            line,
            sensors_referenced: Vec::new(),
        }
    }
}
pub type Setup = String;
pub type UnauthenticatedAction = String;

#[id]
pub struct SensorId;

impl SensorId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Getters, Debug, Clone)]
pub struct Sensor {
    #[copy]
    id: SensorId,
    #[copy]
    prototype_id: SensorPrototypeId,
}

impl Sensor {
    pub async fn new(txn: &mut Transaction<'_>, mut new_sensor: NewSensor) -> Result<Self> {
        let mut uniq = HashSet::new();
        for c in &mut new_sensor.configs {
            uniq.insert(c.request_id());
        }
        if uniq.len() != new_sensor.configs.len() {
            return Err(Error::DuplicatedConfig);
        }

        for config in &new_sensor.configs {
            let mut queue = VecDeque::from_iter(vec![config.value()]);
            while let Some(value) = queue.pop_front() {
                if let Val::Map(vec) = value {
                    let mut uniq = HashSet::new();
                    for c in vec {
                        uniq.insert(c.key());
                        queue.push_back(c.key())
                    }
                    if uniq.len() != vec.len() {
                        return Err(Error::DuplicatedKey);
                    }
                }
            }
        }

        new_sensor
            .configs
            .sort_by(|a, b| a.request_id().cmp(&b.request_id()));
        let serialized = new_sensor
            .configs
            .iter()
            .map(|c| format!("{}-{}", c.request_id(), c.value().to_string()))
            .collect::<Vec<_>>()
            .join(",");

        // TODO: move this to array matching, string has injection risks
        let id: Option<(SensorId,)> = sqlx::query_as(
            "
            SELECT sensor_id
            FROM (SELECT sensor_id, string_agg(concat, ',') as str, COUNT(*) as count
                  FROM (SELECT concat(request_id, '-', value) as concat, sensor_id
                        FROM sensor_configs) as conf
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
                let request =
                    SensorConfigRequest::find_by_id(&mut *txn, config.request_id()).await?;
                SensorConfig::new(&mut *txn, &sensor, &request, config.value().to_string()).await?;
            }
            sensor
        };
        Ok(sensor)
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
        sensor_id: SensorId,
    ) -> Result<Self> {
        let sensor = sqlx::query_as(
            "SELECT id, prototype_id
             FROM sensors
             INNER JOIN sensor_belongs_to_compiler bt ON bt.sensor_id = sensors.id
             WHERE id = $1 AND compiler_id = $2",
        )
        .bind(sensor_id)
        .bind(compiler.id())
        .fetch_one(txn)
        .await?;
        Ok(sensor)
    }

    pub async fn prototype(&self, txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
        SensorPrototype::find_by_id(txn, self.prototype_id).await
    }

    pub async fn configs(&self, txn: &mut Transaction<'_>) -> Result<Vec<SensorConfig>> {
        SensorConfig::find_by_sensor(txn, self).await
    }
}
