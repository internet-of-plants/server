use crate::{Result, Transaction};
use derive::id;
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

#[id]
pub struct DeviceConfigTypeId;

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
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        widget_kind: DeviceWidgetKind,
    ) -> Result<Self> {
        sqlx::query(
            "INSERT INTO device_config_types
            (name, widget)
            VALUES ($1, $2)
            ON CONFLICT (name, widget) DO NOTHING",
        )
        .bind(&name)
        .bind(widget_kind)
        .execute(&mut *txn)
        .await?;

        let (id,): (DeviceConfigTypeId,) =
            sqlx::query_as("SELECT id FROM device_config_types WHERE name = $1 AND widget = $2")
                .bind(&name)
                .bind(widget_kind)
                .fetch_one(&mut *txn)
                .await?;

        Ok(Self {
            id,
            name,
            widget: widget_kind,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: DeviceConfigTypeId) -> Result<Self> {
        let ty = sqlx::query_as("SELECT id, name, widget FROM device_config_types WHERE id = $1")
            .bind(id)
            .fetch_one(txn)
            .await?;
        Ok(ty)
    }
}
