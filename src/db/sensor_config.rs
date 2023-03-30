use crate::{Result, Sensor, SensorConfigRequest, SensorConfigRequestId, SensorId, Transaction};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Getters, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    key: Val,
    value: Val,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Val {
    String(String),
    Number(usize),
    Map(Vec<Element>),
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Val::Number(number) => write!(f, "{}", number)?,
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

#[derive(Getters, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSensorConfig {
    #[copy]
    request_id: SensorConfigRequestId,
    pub value: Val, // encoded the way it will be used by C++, or a map that becomes a array of std::pair<Key, Value>
}

#[derive(Getters, Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SensorConfigView {
    #[copy]
    request_id: SensorConfigRequestId,
    name: String,
    type_name: Option<String>,
    value: String,
}

impl SensorConfigView {
    pub async fn new(txn: &mut Transaction<'_>, config: SensorConfig) -> Result<Self> {
        let request = config.request(txn).await?;
        Ok(Self {
            request_id: config.request_id,
            type_name: request.ty(txn).await?.name().clone(),
            name: request.name().to_owned(),
            value: config.value,
        })
    }
}

#[id]
pub struct SensorConfigId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfig {
    #[copy]
    id: SensorConfigId,
    #[copy]
    sensor_id: SensorId,
    #[copy]
    request_id: SensorConfigRequestId,
    value: String,
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
