use crate::db::target::{Target, TargetId};
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct ConfigTypeId(i64);

impl ConfigTypeId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum WidgetKind {
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    PinSelection,
    Selection(Vec<String>),
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum WidgetKindRaw {
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    PinSelection,
    Selection,
}

impl From<&WidgetKind> for WidgetKindRaw {
    fn from(kind: &WidgetKind) -> Self {
        match kind {
            WidgetKind::U8 => WidgetKindRaw::U8,
            WidgetKind::U16 => WidgetKindRaw::U16,
            WidgetKind::U32 => WidgetKindRaw::U32,
            WidgetKind::U64 => WidgetKindRaw::U64,
            WidgetKind::F32 => WidgetKindRaw::F32,
            WidgetKind::F64 => WidgetKindRaw::F64,
            WidgetKind::String => WidgetKindRaw::String,
            WidgetKind::PinSelection => WidgetKindRaw::PinSelection,
            WidgetKind::Selection(_) => WidgetKindRaw::Selection,
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConfigType {
    id: ConfigTypeId,
    pub name: String,
    widget: WidgetKindRaw,
}

impl ConfigType {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> ConfigTypeId {
        self.id
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: ConfigTypeId) -> Result<Self> {
        let ty = sqlx::query_as("SELECT id, name, widget FROM config_types WHERE id = $1")
            .bind(&id)
            .fetch_one(txn)
            .await?;
        Ok(ty)
    }

    pub async fn widget(&self, txn: &mut Transaction<'_>, target_ids: &[TargetId]) -> Result<WidgetKind> {
        Ok(match &self.widget {
            WidgetKindRaw::U8 => WidgetKind::U8,
            WidgetKindRaw::U16 => WidgetKind::U16,
            WidgetKindRaw::U32 => WidgetKind::U32,
            WidgetKindRaw::U64 => WidgetKind::U64,
            WidgetKindRaw::F32 => WidgetKind::F32,
            WidgetKindRaw::F64 => WidgetKind::F64,
            WidgetKindRaw::String => WidgetKind::String,
            WidgetKindRaw::PinSelection => {
                let mut targets = Vec::new();
                let mut first = true;
                for id in target_ids {
                    let target = Target::find_by_id(txn, *id).await?;
                    let pins = target.board(txn).await?.pins(txn).await?;
                    if first {
                        first = false;
                        targets.extend(pins);
                    } else {
                        targets.retain(|p| pins.contains(p));
                    }
                }
                WidgetKind::Selection(targets)
            }
            WidgetKindRaw::Selection => {
                let options = sqlx::query_as::<_, (String,)>(
                    "SELECT option FROM config_type_selection_options WHERE type_id = $1",
                )
                .bind(&self.id)
                .fetch_all(txn)
                .await?
                .into_iter()
                .map(|(opt,)| opt)
                .collect();
                WidgetKind::Selection(options)
            }
        })
    }

    pub async fn new(txn: &mut Transaction<'_>, name: String, widget: WidgetKind) -> Result<Self> {
        let widget_kind_raw = WidgetKindRaw::from(&widget);
        let (id,) = sqlx::query_as::<_, (ConfigTypeId,)>(
            "INSERT INTO config_types (name, widget) VALUES ($1, $2) RETURNING id",
        )
        .bind(&name)
        .bind(&widget_kind_raw)
        .fetch_one(&mut *txn)
        .await?;

        match widget {
            WidgetKind::Selection(options) => {
                for option in options {
                    sqlx::query(
                        "INSERT INTO config_type_selection_options (type_id, option) VALUES ($1, $2)",
                    )
                        .bind(&id)
                        .bind(&option)
                        .execute(&mut *txn)
                        .await?;
                }
            }
            _ => {}
        }
        Ok(Self {
            id,
            name,
            widget: widget_kind_raw,
        })
    }
}
