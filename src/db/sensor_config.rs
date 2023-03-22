use crate::{Result, Sensor, SensorConfigRequest, SensorConfigRequestId, SensorId, Transaction};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    pub key: Val,
    pub value: Val,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Val {
    String(String),
    Map(Vec<Element>),
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::String(string) => write!(f, "{}", string.replace("\"", "\\\""))?,
            Val::Map(vec) => {
                write!(f, "{{")?;
                for el in vec.iter() {
                    write!(
                        f,
                        "\n  std::make_pair({}, {}),",
                        el.key.to_string(),
                        el.value.to_string()
                    )?;
                }
                write!(f, "}}")?
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSensorConfig {
    pub request_id: SensorConfigRequestId,
    pub value: Val, // encoded the way it will be used by C++, or a map that becomes a array of std::pair<Key, Value>
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SensorConfigView {
    pub request_id: SensorConfigRequestId,
    name: String,
    type_name: String,
    pub value: String,
}

impl SensorConfigView {
    pub async fn new(txn: &mut Transaction<'_>, config: SensorConfig) -> Result<Self> {
        let request = config.request(txn).await?;
        Ok(Self {
            request_id: config.request_id,
            type_name: request.ty(txn).await?.name,
            name: request.name,
            value: config.value,
        })
    }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct SensorConfigId(i64);

impl SensorConfigId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfig {
    pub id: SensorConfigId,
    pub sensor_id: SensorId,
    pub request_id: SensorConfigRequestId,
    pub value: String,
}

impl SensorConfig {
    pub async fn new(
        txn: &mut Transaction<'_>,
        sensor: &Sensor,
        request: &SensorConfigRequest,
        value: String,
    ) -> Result<Self> {
        let (id,) = sqlx::query_as::<_, (SensorConfigId,)>(
            "INSERT INTO sensor_configs (request_id, sensor_id, value) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(request.id())
            .bind(sensor.id())
            .bind(&value)
            .fetch_one(&mut *txn)
            .await?;
        Ok(Self {
            id,
            sensor_id: sensor.id(),
            request_id: request.id(),
            value,
        })
    }

    pub async fn find_by_sensor(txn: &mut Transaction<'_>, sensor: &Sensor) -> Result<Vec<Self>> {
        let list: Vec<Self> = sqlx::query_as(
            "SELECT id, sensor_id, request_id, value FROM sensor_configs WHERE sensor_id = $1",
        )
        .bind(sensor.id())
        .fetch_all(txn)
        .await?;
        Ok(list)
    }

    pub async fn request(&self, txn: &mut Transaction<'_>) -> Result<SensorConfigRequest> {
        SensorConfigRequest::find_by_id(&mut *txn, self.request_id).await
    }
}
