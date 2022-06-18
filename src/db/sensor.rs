pub mod config;
pub mod config_request;
pub mod config_type;

use std::collections::HashSet;

use crate::db::sensor::config::Config;
use crate::db::sensor::config_request::ConfigRequestId;
use crate::db::sensor_prototype::*;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSensor {
    pub prototype_id: SensorPrototypeId,
    pub configs: Vec<NewConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewConfig {
    pub request_id: ConfigRequestId,
    pub value: String, // encoded the way it will be used by C++
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MeasurementType {
    FloatCelsius,
    Percentage,
    RawAnalogRead, // (0-1024)
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Measurement {
    pub name: String,
    pub value: String,
    pub ty: MeasurementType
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
    pub async fn find_by_id(txn: &mut Transaction<'_>, sensor_id: SensorId) -> Result<Self> {
        let sensor: Self = sqlx::query_as("SELECT id, prototype_id FROM sensors WHERE id = $1")
            .bind(&sensor_id)
            .fetch_one(&mut *txn)
            .await?;
        Ok(sensor)
    }

    pub async fn prototype(&self, txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
        SensorPrototype::find_by_id(txn, self.prototype_id).await
    }

    pub async fn configs(&self, txn: &mut Transaction<'_>) -> Result<Vec<Config>> {
        Config::find_by_sensor(txn, self.id).await
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        mut new_sensor: NewSensor,
    ) -> Result<Self> {
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

        let id = if let Some((id,)) = id {
            id
        } else {
            let (id,): (SensorId,) =
                sqlx::query_as("INSERT INTO sensors (prototype_id) VALUES ($1) RETURNING id")
                    .bind(&new_sensor.prototype_id)
                    .fetch_one(&mut *txn)
                    .await?;

            for config in new_sensor.configs {
                Config::new(&mut *txn, id, config.request_id, config.value).await?;
            }
            id
        };
        Ok(Self {
            id,
            prototype_id: new_sensor.prototype_id,
        })
    }

    pub fn id(&self) -> SensorId {
        self.id
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        let sensors: Vec<Self> = sqlx::query_as("SELECT id, prototype_id FROM sensors")
            .fetch_all(&mut *txn)
            .await?;
        Ok(sensors)
    }

    pub async fn list_for_prototype(
        txn: &mut Transaction<'_>,
        prototype_id: SensorPrototypeId,
    ) -> Result<Vec<Self>> {
        let sensors: Vec<Self> =
            sqlx::query_as("SELECT id, prototype_id FROM sensors WHERE prototype_id = $1")
                .bind(&prototype_id)
                .fetch_all(&mut *txn)
                .await?;
        Ok(sensors)
    }
}
