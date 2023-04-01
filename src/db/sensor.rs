use crate::{
    db::sensor_config::Val, Compiler, Dependency, Error, NewSensorConfig, Result, SensorConfig,
    SensorConfigRequest, SensorConfigView, SensorMeasurementView, SensorPrototype,
    SensorPrototypeId, SensorPrototypeView, SensorWidgetKindView, Target, Transaction, ValRaw,
};
use derive::id;
use derive_get::Getters;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
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
    local_pk: u64,
}

impl NewSensor {
    pub fn configs_mut(&mut self) -> &mut [NewSensorConfig] {
        &mut self.configs
    }

    pub async fn sort(
        txn: &mut Transaction<'_>,
        sensors: &[Self],
        targets: &[&Target],
    ) -> Result<Vec<Self>> {
        let sensor_by_local_pk: HashMap<u64, NewSensor> =
            sensors.iter().map(|s| (s.local_pk(), s.clone())).collect();
        let mut graph: HashMap<u64, HashSet<u64>> = HashMap::new();

        for sensor in sensor_by_local_pk.values() {
            let value = graph.entry(sensor.local_pk()).or_default();
            for config in sensor.configs() {
                let request = SensorConfigRequest::find_by_id(txn, config.request_id()).await?;
                let ty = request.ty(txn).await?;
                if let SensorWidgetKindView::Sensor(_) = ty.widget(txn, targets).await? {
                    match config.value() {
                        ValRaw::Integer(number) => {
                            value.insert(*number);
                        }
                        raw => return Err(Error::InvalidValForSensor(raw.clone())),
                    }
                }
            }
        }

        let mut sorted_sensors = Vec::new();
        let mut work_queue: VecDeque<u64> = graph
            .iter()
            .filter(|(_, c)| c.is_empty())
            .map(|(p, _)| *p)
            .collect();

        while let Some(local_pk) = work_queue.pop_front() {
            let sensor = sensor_by_local_pk.get(&local_pk).ok_or_else(|| {
                Error::NewSensorReferencedDoesntExist(
                    local_pk,
                    sensor_by_local_pk.keys().copied().collect(),
                )
            })?;
            sorted_sensors.push(sensor.clone());

            graph.remove(&sensor.local_pk());

            for (parent, children) in graph.iter_mut() {
                let was_empty = children.is_empty();
                children.retain(|el| *el != sensor.local_pk());
                if !was_empty && children.is_empty() {
                    work_queue.push_back(*parent);
                }
            }
        }

        Ok(sorted_sensors)
    }

    pub async fn normalize(
        &mut self,
        txn: &mut Transaction<'_>,
        sensor_by_local_pk: &mut HashMap<u64, Sensor>,
        targets: &[&Target],
    ) -> Result<()> {
        for config in self.configs_mut() {
            let request = SensorConfigRequest::find_by_id(txn, config.request_id()).await?;
            let ty = request.ty(txn).await?;

            if let SensorWidgetKindView::Sensor(prototype_id) = ty.widget(txn, targets).await? {
                let prototype = SensorPrototype::find_by_id(txn, prototype_id).await?;

                if prototype.variable_name().is_none() {
                    return Err(Error::NoVariableNameForReferencedSensor(prototype.id()));
                }

                match &mut config.value {
                    ValRaw::Integer(number) => match sensor_by_local_pk.get(number) {
                        Some(sensor) => *number = i64::from(sensor.id()) as u64,
                        None => return Err(Error::SensorReferencedNotFound(*number, self.clone())),
                    },
                    raw => return Err(Error::InvalidValForSensor(raw.clone())),
                }
            }
        }
        Ok(())
    }
}

#[derive(Getters, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SensorView {
    #[copy]
    id: SensorId,
    name: String,
    #[copy]
    index: i64,
    alias: String,
    variable_name: Option<String>,
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
        let sensors_metadata: Vec<(SensorId, i64, SensorPrototypeId, String, String)> = sqlx::query_as(
            "SELECT DISTINCT ON (sensors.id) sensors.id, sensors.index, sensors.prototype_id, bt.alias, bt.color
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
        for (id, index, prototype_id, alias, color) in sensors_metadata {
            let sensor = Sensor {
                id,
                prototype_id,
                index,
            };
            let prototype = sensor.prototype(txn).await?;
            let sensor_configs = sensor.configs(txn).await?;

            let mut configurations = Vec::with_capacity(sensor_configs.len());
            for config in sensor_configs {
                configurations.push(SensorConfigView::new(txn, config, &[&target]).await?);
            }

            sensors.push(Self {
                id: sensor.id(),
                name: prototype.name().to_owned(),
                index,
                variable_name: prototype.variable_name().clone(),
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
                prototype: SensorPrototypeView::new(txn, prototype.clone(), &[&target]).await?,
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

#[derive(sqlx::FromRow, Getters, Debug, Clone)]
pub struct Sensor {
    #[copy]
    id: SensorId,
    #[copy]
    prototype_id: SensorPrototypeId,
    #[copy]
    index: i64,
}

impl Sensor {
    pub async fn new(
        txn: &mut Transaction<'_>,
        mut new_sensor: NewSensor,
        index: i64,
        targets: &[&Target],
    ) -> Result<Self> {
        let mut uniq = HashSet::new();
        for c in &mut new_sensor.configs {
            uniq.insert(c.request_id());
        }
        if uniq.len() != new_sensor.configs.len() {
            return Err(Error::DuplicatedConfig);
        }

        // Deduplicates new sensors
        for config in &new_sensor.configs {
            let mut queue = VecDeque::from_iter(vec![config.value()]);
            while let Some(value) = queue.pop_front() {
                if let ValRaw::Map(vec) = value {
                    let mut uniq = Vec::new();
                    for c in vec {
                        if uniq.contains(&c.key()) {
                            return Err(Error::DuplicatedKey);
                        }
                        uniq.push(c.key());
                        queue.push_back(c.key());
                    }
                }
            }
        }

        new_sensor
            .configs
            .sort_by_key(|a| a.request_id());
        let mut serialized = Vec::with_capacity(new_sensor.configs.len());
        for c in &new_sensor.configs {
            let request = SensorConfigRequest::find_by_id(&mut *txn, c.request_id()).await?;
            let widget = request.ty(txn).await?.widget(txn, targets).await?;
            let val = Val::new(txn, c.value().clone(), widget.clone()).await?;
            serialized.push(format!(
                "{}-{}",
                c.request_id(),
                val.compile(txn, widget).await?
            ));
        }
        let serialized = serialized.join(",");

        // TODO: move this to array matching, string has injection risks
        let id: Option<(SensorId,)> = dbg!(
            sqlx::query_as(
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
            .bind(new_sensor.prototype_id)
            .bind(new_sensor.configs.len() as i64)
            .bind(&serialized)
            .fetch_optional(&mut *txn)
            .await
        )?;

        let sensor = if let Some((id,)) = id {
            Self {
                id,
                index,
                prototype_id: new_sensor.prototype_id,
            }
        } else {
            let (id,): (SensorId,) = sqlx::query_as(
                "INSERT INTO sensors (prototype_id, index) VALUES ($1, $2) RETURNING id",
            )
            .bind(new_sensor.prototype_id)
            .bind(index)
            .fetch_one(&mut *txn)
            .await?;

            let sensor = Self {
                id,
                index,
                prototype_id: new_sensor.prototype_id,
            };
            for config in new_sensor.configs {
                let request =
                    SensorConfigRequest::find_by_id(&mut *txn, config.request_id()).await?;
                let widget = request.ty(txn).await?.widget(txn, targets).await?;
                let val = Val::new(txn, config.value().clone(), widget).await?;
                SensorConfig::new(&mut *txn, &sensor, &request, val).await?;
            }
            sensor
        };
        Ok(sensor)
    }

    pub async fn raw_find_by_id(txn: &mut Transaction<'_>, sensor_id: SensorId) -> Result<Self> {
        let sensor = sqlx::query_as(
            "SELECT id, prototype_id, index
             FROM sensors
             WHERE id = $1",
        )
        .bind(sensor_id)
        .fetch_one(txn)
        .await?;
        Ok(sensor)
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
        sensor_id: SensorId,
    ) -> Result<Self> {
        let sensor = sqlx::query_as(
            "SELECT id, prototype_id, index
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
