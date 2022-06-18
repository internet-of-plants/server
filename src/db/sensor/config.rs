use crate::db::sensor::config_request::{ConfigRequest, ConfigRequestId};
use crate::db::sensor::SensorId;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct ConfigId(i64);

impl ConfigId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Config {
    pub id: ConfigId,
    pub sensor_id: SensorId,
    pub request_id: ConfigRequestId,
    pub value: String,
}

impl Config {
    pub async fn request(&self, txn: &mut Transaction<'_>) -> Result<ConfigRequest> {
        Ok(ConfigRequest::find_by_id(&mut *txn, self.request_id).await?)
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        sensor_id: SensorId,
        request_id: ConfigRequestId,
        value: String,
    ) -> Result<Self> {
        let (id,) = sqlx::query_as::<_, (ConfigId,)>(
            "INSERT INTO configs (request_id, sensor_id, value) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&request_id)
        .bind(&sensor_id)
        .bind(&value)
        .fetch_one(&mut *txn)
        .await?;
        Ok(Self {
            id,
            sensor_id,
            request_id,
            value,
        })
    }

    pub async fn find_by_sensor(
        txn: &mut Transaction<'_>,
        sensor_id: SensorId,
    ) -> Result<Vec<Self>> {
        let list: Vec<Self> = sqlx::query_as(
            "SELECT id, sensor_id, request_id, value FROM configs WHERE sensor_id = $1"
        )
            .bind(&sensor_id)
            .fetch_all(txn)
            .await?;
        Ok(list)
    }
}
