use crate::{Result, Target, Transaction};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfigTypeView {
    pub name: String,
    pub widget: SensorWidgetKind,
}

impl SensorConfigTypeView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        ty: SensorConfigType,
        targets: &[&Target],
    ) -> Result<Self> {
        Ok(Self {
            name: ty.name().to_owned(),
            widget: ty.widget(txn, targets).await?,
        })
    }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct SensorConfigTypeId(i64);

impl SensorConfigTypeId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum SensorWidgetKind {
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
pub enum SensorWidgetKindRaw {
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

impl From<&SensorWidgetKind> for SensorWidgetKindRaw {
    fn from(kind: &SensorWidgetKind) -> Self {
        match kind {
            SensorWidgetKind::U8 => SensorWidgetKindRaw::U8,
            SensorWidgetKind::U16 => SensorWidgetKindRaw::U16,
            SensorWidgetKind::U32 => SensorWidgetKindRaw::U32,
            SensorWidgetKind::U64 => SensorWidgetKindRaw::U64,
            SensorWidgetKind::F32 => SensorWidgetKindRaw::F32,
            SensorWidgetKind::F64 => SensorWidgetKindRaw::F64,
            SensorWidgetKind::String => SensorWidgetKindRaw::String,
            SensorWidgetKind::PinSelection => SensorWidgetKindRaw::PinSelection,
            SensorWidgetKind::Selection(_) => SensorWidgetKindRaw::Selection,
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfigType {
    id: SensorConfigTypeId,
    pub name: String,
    widget: SensorWidgetKindRaw,
}

impl SensorConfigType {
    pub async fn find_by_id(txn: &mut Transaction<'_>, id: SensorConfigTypeId) -> Result<Self> {
        let ty = sqlx::query_as("SELECT id, name, widget FROM sensor_config_types WHERE id = $1")
            .bind(&id)
            .fetch_one(txn)
            .await?;
        Ok(ty)
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        widget: SensorWidgetKind,
    ) -> Result<Self> {
        let widget_kind_raw = SensorWidgetKindRaw::from(&widget);
        let (id,) = sqlx::query_as::<_, (SensorConfigTypeId,)>(
            "INSERT INTO sensor_config_types (name, widget) VALUES ($1, $2) RETURNING id",
        )
        .bind(&name)
        .bind(&widget_kind_raw)
        .fetch_one(&mut *txn)
        .await?;

        if let SensorWidgetKind::Selection(options) = widget {
            for option in options {
                sqlx::query(
                    "INSERT INTO sensor_config_type_selection_options (type_id, option) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(&id)
                .bind(&option)
                .execute(&mut *txn)
                .await?;
            }
        }
        Ok(Self {
            id,
            name,
            widget: widget_kind_raw,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn id(&self) -> SensorConfigTypeId {
        self.id
    }

    pub async fn widget(
        &self,
        txn: &mut Transaction<'_>,
        targets: &[&Target],
    ) -> Result<SensorWidgetKind> {
        Ok(match &self.widget {
            SensorWidgetKindRaw::U8 => SensorWidgetKind::U8,
            SensorWidgetKindRaw::U16 => SensorWidgetKind::U16,
            SensorWidgetKindRaw::U32 => SensorWidgetKind::U32,
            SensorWidgetKindRaw::U64 => SensorWidgetKind::U64,
            SensorWidgetKindRaw::F32 => SensorWidgetKind::F32,
            SensorWidgetKindRaw::F64 => SensorWidgetKind::F64,
            SensorWidgetKindRaw::String => SensorWidgetKind::String,
            SensorWidgetKindRaw::PinSelection => {
                let mut pins = Vec::new();
                let mut first = true;
                for target in targets {
                    let p = target.pins(txn).await?;
                    if first {
                        first = false;
                        pins.extend(p);
                    } else {
                        pins.retain(|p| p.contains(p));
                    }
                }
                SensorWidgetKind::Selection(pins)
            }
            SensorWidgetKindRaw::Selection => {
                let options = sqlx::query_as::<_, (String,)>(
                    "SELECT option FROM sensor_config_type_selection_options WHERE type_id = $1",
                )
                .bind(&self.id)
                .fetch_all(txn)
                .await?
                .into_iter()
                .map(|(opt,)| opt)
                .collect();
                SensorWidgetKind::Selection(options)
            }
        })
    }
}
