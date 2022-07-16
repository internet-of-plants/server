use crate::{
    Compiler, DeviceConfigRequest, DeviceConfigRequestId, Organization, Result, Transaction,
};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewDeviceConfig {
    pub request_id: DeviceConfigRequestId,
    pub value: String, // encoded the way it will be used by C++
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceConfigView {
    pub request_id: DeviceConfigRequestId,
    name: String,
    type_name: String,
    pub value: String,
}

impl DeviceConfigView {
    pub async fn new(txn: &mut Transaction<'_>, config: DeviceConfig) -> Result<Self> {
        let request = config.request(&mut *txn).await?;
        Ok(Self {
            request_id: config.request_id,
            type_name: request.ty(&mut *txn).await?.name,
            name: request.name,
            value: config.value,
        })
    }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct DeviceConfigId(pub i64);

impl DeviceConfigId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeviceConfig {
    pub id: DeviceConfigId,
    pub request_id: DeviceConfigRequestId,
    pub value: String,
}

impl DeviceConfig {
    pub async fn new(
        txn: &mut Transaction<'_>,
        new_config: NewDeviceConfig,
        organization: &Organization,
    ) -> Result<Self> {
        let id = sqlx::query_as("SELECT id FROM device_configs WHERE request_id = $1 AND value = $2 AND organization_id = $3")
            .bind(new_config.request_id)
            .bind(&new_config.value)
            .bind(organization.id())
            .fetch_optional(&mut *txn)
            .await?;

        let id = match id {
            Some((id,)) => id,
            None => {
                let (id,) = sqlx::query_as(
                    "INSERT INTO device_configs (request_id, value, organization_id) VALUES ($1, $2, $3) RETURNING id",
                )
                    .bind(new_config.request_id)
                    .bind(&new_config.value)
                    .bind(organization.id())
                    .fetch_one(&mut *txn)
                    .await?;
                id
            }
        };

        Ok(Self {
            id,
            request_id: new_config.request_id,
            value: new_config.value,
        })
    }

    pub async fn find_by_compiler(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
    ) -> Result<Vec<Self>> {
        let list: Vec<Self> = sqlx::query_as(
            "SELECT id, request_id, value
            FROM device_configs
            INNER JOIN device_config_belongs_to_compiler bt ON bt.config_id = device_configs.id
            WHERE compiler_id = $1",
        )
        .bind(compiler.id())
        .fetch_all(txn)
        .await?;
        Ok(list)
    }

    pub async fn request(&self, txn: &mut Transaction<'_>) -> Result<DeviceConfigRequest> {
        DeviceConfigRequest::find_by_id(&mut *txn, self.request_id).await
    }

    pub fn id(&self) -> DeviceConfigId {
        self.id
    }
}
