use crate::{
    DeviceConfigType, DeviceConfigTypeId, DeviceConfigTypeView, DeviceWidgetKind, Result, Target,
    Transaction,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfigRequestView {
    #[copy]
    id: DeviceConfigRequestId,
    variable_name: String,
    name: String,
    ty: DeviceConfigTypeView,
}

impl DeviceConfigRequestView {
    pub async fn new(txn: &mut Transaction<'_>, request: DeviceConfigRequest) -> Result<Self> {
        let ty = request.ty(txn).await?;
        Ok(Self {
            id: request.id(),
            variable_name: request.variable_name().to_owned(),
            name: request.name().to_owned(),
            ty: DeviceConfigTypeView::new(ty),
        })
    }
}

#[id]
pub struct DeviceConfigRequestId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewDeviceConfigRequest {
    pub variable_name: String,
    pub name: String,
    pub type_name: String,
    #[copy]
    pub widget: DeviceWidgetKind,
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeviceConfigRequest {
    #[copy]
    id: DeviceConfigRequestId,
    variable_name: String,
    name: String,
    #[copy]
    type_id: DeviceConfigTypeId,
}

impl DeviceConfigRequest {
    pub async fn new(
        txn: &mut Transaction<'_>,
        request: &NewDeviceConfigRequest,
        target: &Target,
    ) -> Result<Self> {
        let ty =
            DeviceConfigType::new(txn, request.type_name().to_owned(), request.widget()).await?;

        sqlx::query(
            "INSERT INTO device_config_requests
            (type_id, variable_name, name, target_id)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (type_id, variable_name, target_id)
            DO UPDATE SET name = $3",
        )
        .bind(ty.id())
        .bind(request.variable_name())
        .bind(request.name())
        .bind(target.id())
        .execute(&mut *txn)
        .await?;

        let (id,): (DeviceConfigRequestId,) = sqlx::query_as(
            "SELECT id FROM device_config_requests WHERE type_id = $1 AND variable_name = $2 AND target_id = $3"
        )
            .bind(ty.id())
            .bind(request.variable_name())
            .bind(target.id())
            .fetch_one(txn)
            .await?;
        Ok(Self {
            id,
            type_id: ty.id(),
            variable_name: request.variable_name().to_owned(),
            name: request.name().to_owned(),
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: DeviceConfigRequestId) -> Result<Self> {
        let request = sqlx::query_as(
            "SELECT id, type_id, variable_name, name, target_id FROM device_config_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_one(txn)
        .await?;
        Ok(request)
    }

    pub async fn find_by_target(txn: &mut Transaction<'_>, target: &Target) -> Result<Vec<Self>> {
        let requests = sqlx::query_as(
            "SELECT id, type_id, variable_name, name, target_id FROM device_config_requests WHERE target_id = $1",
        )
            .bind(target.id())
            .fetch_all(txn)
            .await?;
        Ok(requests)
    }

    pub async fn ty(&self, txn: &mut Transaction<'_>) -> Result<DeviceConfigType> {
        DeviceConfigType::find_by_id(txn, self.type_id).await
    }
}
