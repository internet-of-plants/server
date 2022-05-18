pub mod config;
pub mod config_request;
pub mod config_type;

use crate::db::sensor::config::Config;
use crate::db::sensor::config_request::ConfigRequestId;
use crate::db::sensor_prototype::*;
use crate::db::user::UserId;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSensor {
    pub prototype_id: SensorPrototypeId,
    pub configs: Vec<NewConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewConfig {
    pub request_id: ConfigRequestId,
    pub value: String, // encoded the way it will be used by C++
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Measurement {
    pub name: String,
    pub value: String,
}

pub type Dependency = String;
pub type Include = String;
pub type Definition = String;
pub type Setup = String;

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct SensorId(i64);

impl SensorId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Sensor {
    pub id: SensorId,
    pub owner_id: UserId,
    pub prototype_id: SensorPrototypeId,
}

impl Sensor {
    pub async fn find_by_id(txn: &mut Transaction<'_>, sensor_id: SensorId) -> Result<Self> {
        let sensor: Self =
            sqlx::query_as("SELECT id, owner_id, prototype_id FROM sensors WHERE id = $1")
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
        owner_id: UserId,
        new_sensor: NewSensor,
    ) -> Result<Self> {
        let (id,): (SensorId,) = sqlx::query_as(
            "INSERT INTO sensors (owner_id, prototype_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(&owner_id)
        .bind(&new_sensor.prototype_id)
        .fetch_one(&mut *txn)
        .await?;

        let mut configs = Vec::with_capacity(new_sensor.configs.len());
        for config in new_sensor.configs {
            let req_id = config.request_id;
            let config = Config::new(&mut *txn, owner_id, id, req_id, config.value).await?;
            configs.push(config);
        }
        Ok(Self {
            id,
            owner_id,
            prototype_id: new_sensor.prototype_id,
        })
    }

    pub fn id(&self) -> SensorId {
        self.id
    }

    pub async fn list(txn: &mut Transaction<'_>, owner_id: UserId) -> Result<Vec<Self>> {
        let sensors: Vec<Self> =
            sqlx::query_as("SELECT id, owner_id, prototype_id FROM sensors WHERE owner_id = $1")
                .bind(&owner_id)
                .fetch_all(&mut *txn)
                .await?;
        Ok(sensors)
    }

    pub async fn list_for_prototype(
        txn: &mut Transaction<'_>,
        owner_id: UserId,
        prototype_id: SensorPrototypeId,
    ) -> Result<Vec<Self>> {
        let sensors: Vec<Self> = sqlx::query_as(
            "SELECT id, owner_id, prototype_id FROM sensors WHERE prototype_id = $1 AND owner_id = $2",
        )
        .bind(&prototype_id)
        .bind(&owner_id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(sensors)
    }
}
