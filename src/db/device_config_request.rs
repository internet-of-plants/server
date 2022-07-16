use crate::{
    DeviceConfigType, DeviceConfigTypeId, DeviceConfigTypeView, DeviceWidgetKind, Result,
    SecretAlgo, Target, Transaction,
};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfigRequestView {
    pub id: DeviceConfigRequestId,
    pub name: String,
    pub human_name: String,
    pub optional: bool,
    pub secret_algo: Option<SecretAlgo>,
    pub ty: DeviceConfigTypeView,
}

impl DeviceConfigRequestView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        request: DeviceConfigRequest,
    ) -> Result<Self> {
        let ty = request.ty(txn).await?;
        Ok(Self {
            id: request.id(),
            name: request.name().to_owned(),
            human_name: request.human_name().to_owned(),
            ty: DeviceConfigTypeView::new(ty),
            optional: request.optional,
            secret_algo: request.secret_algo,
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
    Eq,
    FromStr,
    Hash,
    PartialOrd,
    Ord,
)]
#[sqlx(transparent)]
pub struct DeviceConfigRequestId(pub i64);

impl DeviceConfigRequestId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewDeviceConfigRequest {
    pub name: String,
    pub human_name: String,
    pub type_name: String,
    pub optional: bool,
    pub secret_algo: Option<SecretAlgo>,
    pub widget: DeviceWidgetKind,
}

impl NewDeviceConfigRequest {
    pub fn new(
        human_name: String,
        name: String,
        type_name: String,
        widget: DeviceWidgetKind,
        optional: bool,
        secret_algo: Option<SecretAlgo>,
    ) -> Self {
        Self {
            name,
            human_name,
            type_name,
            optional,
            secret_algo,
            widget,
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeviceConfigRequest {
    pub id: DeviceConfigRequestId,
    pub name: String,
    pub human_name: String,
    pub optional: bool,
    pub secret_algo: Option<SecretAlgo>,
    pub type_id: DeviceConfigTypeId,
}

impl DeviceConfigRequest {
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        human_name: String,
        type_name: String,
        widget: DeviceWidgetKind,
        target: &Target,
        optional: bool,
        secret_algo: Option<SecretAlgo>,
    ) -> Result<Self> {
        let ty = DeviceConfigType::new(txn, type_name, widget).await?;

        let (id,) = sqlx::query_as::<_, (DeviceConfigRequestId,)>(
            "INSERT INTO device_config_requests (type_id, name, human_name, target_id, optional, secret_algo) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
            .bind(ty.id())
            .bind(&name)
            .bind(&human_name)
            .bind(target.id())
            .bind(optional)
            .bind(&secret_algo)
            .fetch_one(&mut *txn)
            .await?;
        Ok(Self {
            id,
            type_id: ty.id(),
            name,
            optional,
            secret_algo,
            human_name,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: DeviceConfigRequestId) -> Result<Self> {
        let request = sqlx::query_as(
            "SELECT id, type_id, name, human_name, target_id, optional, secret_algo FROM device_config_requests WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(request)
    }

    pub async fn find_by_target(txn: &mut Transaction<'_>, target: &Target) -> Result<Vec<Self>> {
        let requests = sqlx::query_as(
            "SELECT id, type_id, name, human_name, target_id, optional, secret_algo FROM device_config_requests WHERE target_id = $1",
        )
            .bind(target.id())
            .fetch_all(&mut *txn)
            .await?;
        Ok(requests)
    }

    pub fn id(&self) -> DeviceConfigRequestId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn human_name(&self) -> &str {
        &self.human_name
    }

    pub async fn ty(&self, txn: &mut Transaction<'_>) -> Result<DeviceConfigType> {
        DeviceConfigType::find_by_id(txn, self.type_id).await
    }
}
