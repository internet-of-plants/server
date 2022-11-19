use crate::{
    Compiler, Error, NewSensorConfig, Result, SensorConfig, SensorConfigRequest, SensorConfigView,
    SensorMeasurementView, SensorPrototype, SensorPrototypeId, SensorPrototypeView, Transaction,
    db::sensor_config::Val,
};
use derive_more::FromStr;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{iter::FromIterator, collections::HashSet, collections::VecDeque};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSensor {
    pub prototype_id: SensorPrototypeId,
    pub alias: String,
    pub configs: Vec<NewSensorConfig>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SensorView {
    pub id: SensorId,
    pub name: String,
    pub alias: String,
    pub color: String,
    pub dependencies: Vec<String>,
    pub includes: Vec<String>,
    pub definitions: Vec<String>,
    pub setups: Vec<String>,
    pub unauthenticated_actions: Vec<String>,
    pub measurements: Vec<SensorMeasurementView>,
    pub configurations: Vec<SensorConfigView>,
    pub prototype: SensorPrototypeView,
}

impl SensorView {
    pub async fn list_for_compiler(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
    ) -> Result<Vec<Self>> {
        let sensors_metadata: Vec<(SensorId, SensorPrototypeId, String, String)> = sqlx::query_as(
            "SELECT DISTINCT(sensors.id), sensors.prototype_id, bt.alias, bt.color
             FROM sensors
             INNER JOIN sensor_prototypes ON sensor_prototypes.id = sensors.prototype_id
             INNER JOIN sensor_belongs_to_compiler bt ON bt.sensor_id = sensors.id
             WHERE bt.compiler_id = $1
             ORDER BY sensors.id ASC",
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
                        let name = reg.render_template(&m.name, &json!({ "index": index }))?;
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

pub type Dependency = String;
pub type Include = String;
pub type Definition = String;
pub type Setup = String;
pub type UnauthenticatedAction = String;

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
        for c in &new_sensor.configs {
            uniq.insert(c.request_id);
        }
        if uniq.len() != new_sensor.configs.len() {
            return Err(Error::DuplicatedConfig);
        }

        for config in &new_sensor.configs {
            let mut queue = VecDeque::from_iter(vec![&config.value]);
            while let Some(value) = queue.pop_front() { 
                if let Val::Map(vec) = value {
                    let mut uniq = HashSet::new();
                    for c in vec {
                        uniq.insert(&c.key);
                        queue.push_back(&c.key)
                    }
                    if uniq.len() != vec.len() {
                        return Err(Error::DuplicatedKey);
                    }
                }
            }
        }

        new_sensor
            .configs
            .sort_by(|a, b| a.request_id.cmp(&b.request_id));
        let serialized = new_sensor
            .configs
            .iter()
            .map(|c| format!("{}-{}", c.request_id.0, c.value.to_string()))
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
                let request = SensorConfigRequest::find_by_id(&mut *txn, config.request_id).await?;
                SensorConfig::new(&mut *txn, &sensor, &request, config.value.to_string()).await?;
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

    pub async fn configs(&self, txn: &mut Transaction<'_>) -> Result<Vec<SensorConfig>> {
        SensorConfig::find_by_sensor(txn, self).await
    }
}
