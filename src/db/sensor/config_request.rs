use crate::db::sensor::config_type::{ConfigType, ConfigTypeId, WidgetKind};
use crate::db::sensor_prototype::SensorPrototypeId;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct ConfigRequestId(i64);

impl ConfigRequestId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewConfigRequest {
    pub name: String,
    pub type_name: String,
    pub widget: WidgetKind,
}

impl NewConfigRequest {
    pub fn new(name: String, type_name: String, widget: WidgetKind) -> Self {
        Self {
            name,
            type_name,
            widget,
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConfigRequest {
    pub id: ConfigRequestId,
    pub name: String,
    pub type_id: ConfigTypeId,
}

impl ConfigRequest {
    pub fn id(&self) -> ConfigRequestId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        type_name: String,
        widget: WidgetKind,
        sensor_prototype_id: SensorPrototypeId,
    ) -> Result<Self> {
        let ty = ConfigType::new(txn, type_name, widget).await?;

        let (id,) = sqlx::query_as::<_, (ConfigRequestId,)>(
            "INSERT INTO config_requests (type_id, name, sensor_prototype_id) VALUES ($1, $2, $3) RETURNING id",
        )
            .bind(ty.id())
            .bind(&name)
            .bind(&sensor_prototype_id)
            .fetch_one(&mut *txn)
            .await?;
        Ok(Self {
            id,
            type_id: ty.id(),
            name,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: ConfigRequestId) -> Result<Self> {
        let request = sqlx::query_as(
            "SELECT id, type_id, name, sensor_prototype_id FROM config_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(request)
    }

    pub async fn list_by_sensor_prototype(
        txn: &mut Transaction<'_>,
        sensor_prototype_id: SensorPrototypeId,
    ) -> Result<Vec<Self>> {
        let list = sqlx::query_as(
            "SELECT id, type_id, name, sensor_prototype_id FROM config_requests WHERE sensor_prototype_id = $1",
        )
        .bind(sensor_prototype_id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list)
    }

    pub async fn ty(&self, txn: &mut Transaction<'_>) -> Result<ConfigType> {
        ConfigType::find_by_id(txn, self.type_id).await
    }
}
