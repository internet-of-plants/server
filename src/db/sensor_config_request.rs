use crate::{
    Result, SensorConfigType, SensorConfigTypeId, SensorConfigTypeView, SensorPrototype,
    SensorWidgetKind, Target, Transaction,
};
use derive_more::{FromStr, Display};
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SensorConfigRequestView {
    #[copy]
    id: SensorConfigRequestId,
    name: String,
    human_name: String,
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
            name: request.name().to_owned(),
            human_name: request.human_name().to_owned(),
            ty: SensorConfigTypeView::new(txn, ty, targets).await?,
        })
    }
}

#[derive(
    Serialize,
    Deserialize,
    sqlx::Type,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Display,
    Eq,
    FromStr,
    Hash,
    PartialOrd,
    Ord,
)]
#[sqlx(transparent)]
pub struct SensorConfigRequestId(pub i64);

impl SensorConfigRequestId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewSensorConfigRequest {
    name: String,
    human_name: String,
    type_name: Option<String>,
    widget: SensorWidgetKind,
}

impl NewSensorConfigRequest {
    pub fn new(
        human_name: String,
        name: String,
        type_name: impl Into<Option<String>>,
        widget: SensorWidgetKind,
    ) -> Self {
        Self {
            name,
            human_name,
            type_name: type_name.into(),
            widget,
        }
    }
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfigRequest {
    #[copy]
    id: SensorConfigRequestId,
    name: String,
    human_name: String,
    type_id: SensorConfigTypeId,
}

impl SensorConfigRequest {
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        human_name: String,
        type_name: Option<String>,
        widget: SensorWidgetKind,
        sensor_prototype: &SensorPrototype,
    ) -> Result<Self> {
        let ty = SensorConfigType::new(txn, type_name, widget).await?;

        let (id,) = sqlx::query_as::<_, (SensorConfigRequestId,)>(
            "INSERT INTO sensor_config_requests (type_id, name, human_name, sensor_prototype_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
            .bind(ty.id())
            .bind(&name)
            .bind(&human_name)
            .bind(sensor_prototype.id())
            .fetch_one(txn)
            .await?;
        Ok(Self {
            id,
            type_id: ty.id(),
            name,
            human_name,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: SensorConfigRequestId) -> Result<Self> {
        let request = sqlx::query_as(
            "SELECT id, type_id, name, human_name, sensor_prototype_id FROM sensor_config_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_one(txn)
        .await?;
        Ok(request)
    }

    pub async fn ty(&self, txn: &mut Transaction<'_>) -> Result<SensorConfigType> {
        Ok(SensorConfigType::find_by_id(txn, self.type_id).await?)
    }
}
