use crate::db::sensor::config_type::{ConfigType, ConfigTypeId, WidgetKind};
use crate::db::sensor_prototype::SensorPrototypeId;
use crate::db::target::Target;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

use super::config_type::ConfigTypeView;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRequestView {
    pub id: ConfigRequestId,
    pub name: String,
    pub human_name: String,
    pub ty: ConfigTypeView,
}

impl ConfigRequestView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        request: ConfigRequest,
        targets: &[&Target],
    ) -> Result<Self> {
        let ty = request.ty(txn).await?;
        Ok(Self {
            id: request.id(),
            name: request.name().to_owned(),
            human_name: request.human_name().to_owned(),
            ty: ConfigTypeView::new(txn, ty, targets).await?,
        })
    }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr, Hash, PartialOrd, Ord)]
#[sqlx(transparent)]
pub struct ConfigRequestId(pub i64);

impl ConfigRequestId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewConfigRequest {
    pub name: String,
    pub human_name: String,
    pub type_name: String,
    pub widget: WidgetKind,
}

impl NewConfigRequest {
    pub fn new(human_name: String, name: String, type_name: String, widget: WidgetKind) -> Self {
        Self {
            name,
            human_name,
            type_name,
            widget,
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConfigRequest {
    pub id: ConfigRequestId,
    pub name: String,
    pub human_name: String,
    pub type_id: ConfigTypeId,
}

impl ConfigRequest {
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        human_name: String,
        type_name: String,
        widget: WidgetKind,
        sensor_prototype_id: SensorPrototypeId,
    ) -> Result<Self> {
        let ty = ConfigType::new(txn, type_name, widget).await?;

        let (id,) = sqlx::query_as::<_, (ConfigRequestId,)>(
            "INSERT INTO config_requests (type_id, name, human_name, sensor_prototype_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
            .bind(ty.id())
            .bind(&name)
            .bind(&human_name)
            .bind(&sensor_prototype_id)
            .fetch_one(&mut *txn)
            .await?;
        Ok(Self {
            id,
            type_id: ty.id(),
            name,
            human_name
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: ConfigRequestId) -> Result<Self> {
        let request = sqlx::query_as(
            "SELECT id, type_id, name, human_name, sensor_prototype_id FROM config_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(request)
    }

    pub fn id(&self) -> ConfigRequestId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn human_name(&self) -> &str {
        &self.human_name
    }

    pub async fn ty(&self, txn: &mut Transaction<'_>) -> Result<ConfigType> {
        ConfigType::find_by_id(txn, self.type_id).await
    }
}
