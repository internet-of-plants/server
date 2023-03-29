use crate::{Result, Transaction};
use derive_more::FromStr;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeviceConfigTypeView {
    name: String,
    #[copy]
    widget: DeviceWidgetKind,
}

impl DeviceConfigTypeView {
    pub fn new(ty: DeviceConfigType) -> Self {
        Self {
            name: ty.name().to_owned(),
            widget: ty.widget(),
        }
    }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct DeviceConfigTypeId(i64);

impl DeviceConfigTypeId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum DeviceWidgetKind {
    SSID,
    PSK,
    Timezone,
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeviceConfigType {
    #[copy]
    id: DeviceConfigTypeId,
    name: String,
    #[copy]
    widget: DeviceWidgetKind,
}

impl DeviceConfigType {
    pub async fn find_by_id(txn: &mut Transaction<'_>, id: DeviceConfigTypeId) -> Result<Self> {
        let ty = sqlx::query_as("SELECT id, name, widget FROM device_config_types WHERE id = $1")
            .bind(&id)
            .fetch_one(txn)
            .await?;
        Ok(ty)
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        widget_kind: DeviceWidgetKind,
    ) -> Result<Self> {
        let (id,) = sqlx::query_as::<_, (DeviceConfigTypeId,)>(
            "INSERT INTO device_config_types (name, widget) VALUES ($1, $2) RETURNING id",
        )
        .bind(&name)
        .bind(&widget_kind)
        .fetch_one(&mut *txn)
        .await?;

        Ok(Self {
            id,
            name,
            widget: widget_kind,
        })
    }
}
