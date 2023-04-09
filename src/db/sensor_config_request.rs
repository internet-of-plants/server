use crate::{
    NewSensorWidgetKind, Result, SensorConfigType, SensorConfigTypeId, SensorConfigTypeView,
    SensorPrototype, Target, Transaction,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SensorConfigRequestView {
    #[copy]
    id: SensorConfigRequestId,
    variable_name: String,
    name: String,
    ty: SensorConfigTypeView,
}

impl SensorConfigRequestView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        request: SensorConfigRequest,
        targets: &[&Target],
    ) -> Result<Self> {
        let ty = request.ty(txn).await?;
        Ok(Self {
            id: request.id(),
            variable_name: request.variable_name().to_owned(),
            name: request.name().to_owned(),
            ty: SensorConfigTypeView::new(txn, ty, targets).await?,
        })
    }
}

#[id]
pub struct SensorConfigRequestId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewSensorConfigRequest {
    variable_name: String,
    name: String,
    #[serde(default)]
    type_name: Option<String>,
    widget: NewSensorWidgetKind,
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfigRequest {
    #[copy]
    id: SensorConfigRequestId,
    variable_name: String,
    name: String,
    type_id: SensorConfigTypeId,
}

impl SensorConfigRequest {
    pub async fn new(
        txn: &mut Transaction<'_>,
        variable_name: String,
        name: String,
        type_name: Option<String>,
        widget: NewSensorWidgetKind,
        sensor_prototype: &SensorPrototype,
    ) -> Result<Self> {
        let ty = SensorConfigType::new(txn, type_name, widget).await?;

        let (id,) = sqlx::query_as::<_, (SensorConfigRequestId,)>(
            "INSERT INTO sensor_config_requests (type_id, variable_name, name, sensor_prototype_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
            .bind(ty.id())
            .bind(&variable_name)
            .bind(&name)
            .bind(sensor_prototype.id())
            .fetch_one(txn)
            .await?;
        Ok(Self {
            id,
            type_id: ty.id(),
            variable_name,
            name,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: SensorConfigRequestId) -> Result<Self> {
        let request = sqlx::query_as(
            "SELECT id, type_id, variable_name, name, sensor_prototype_id FROM sensor_config_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_one(txn)
        .await?;
        Ok(request)
    }

    pub async fn ty(&self, txn: &mut Transaction<'_>) -> Result<SensorConfigType> {
        SensorConfigType::find_by_id(txn, self.type_id).await
    }

    /// A sensor should require 0-N configuration variables to be defined by the user
    pub async fn configuration_requests(
        txn: &mut Transaction<'_>,
        prototype: &SensorPrototype,
    ) -> Result<Vec<Self>> {
        let list = sqlx::query_as(
            "SELECT id, variable_name, name, type_id FROM sensor_config_requests WHERE sensor_prototype_id = $1",
        )
        .bind(prototype.id())
        .fetch_all(&mut *txn)
        .await?;
        Ok(list)
    }
}
