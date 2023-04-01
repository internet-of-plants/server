use crate::{Result, Sensor, SensorConfigRequest, SensorConfigRequestId, SensorId, Transaction, SensorConfigRequestView, Target};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

pub mod val;

pub use val::{Val, ValRaw};

#[derive(Getters, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewSensorConfig {
    #[copy]
    request_id: SensorConfigRequestId,
    pub value: ValRaw,
}

#[derive(Getters, Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SensorConfigView {
    request: SensorConfigRequestView,
    name: String,
    type_name: Option<String>,
    value: Val,
}

impl SensorConfigView {
    pub async fn new(txn: &mut Transaction<'_>, config: SensorConfig, targets: &[&Target]) -> Result<Self> {
        let request = config.request(txn).await?;
        Ok(Self {
            type_name: request.ty(txn).await?.name().clone(),
            name: request.name().to_owned(),
            value: config.value,
            request: SensorConfigRequestView::new(txn, request, targets).await?,
        })
    }
}

#[id]
pub struct SensorConfigId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SensorConfig {
    #[copy]
    id: SensorConfigId,
    #[copy]
    sensor_id: SensorId,
    #[copy]
    request_id: SensorConfigRequestId,
    value: Val,
}

impl SensorConfig {
    pub async fn new(
        txn: &mut Transaction<'_>,
        sensor: &Sensor,
        request: &SensorConfigRequest,
        value: Val,
    ) -> Result<Self> {
        let (id,) = sqlx::query_as::<_, (SensorConfigId,)>(
            "INSERT INTO sensor_configs (request_id, sensor_id, value) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(request.id())
            .bind(sensor.id())
            .bind(&serde_json::to_value(&value)?)
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
        let list: Vec<(
            SensorConfigId,
            SensorId,
            SensorConfigRequestId,
            serde_json::Value,
        )> = sqlx::query_as(
            "SELECT id, sensor_id, request_id, value FROM sensor_configs WHERE sensor_id = $1",
        )
        .bind(sensor.id())
        .fetch_all(txn)
        .await?;
        list.into_iter()
            .map(|(id, sensor_id, request_id, value)| {
                let value = serde_json::from_value(value)?;
                Ok(Self {
                    id,
                    sensor_id,
                    request_id,
                    value,
                })
            })
            .collect::<Result<Vec<Self>>>()
    }

    pub async fn request(&self, txn: &mut Transaction<'_>) -> Result<SensorConfigRequest> {
        SensorConfigRequest::find_by_id(&mut *txn, self.request_id).await
    }
}
