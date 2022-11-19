use crate::{Result, Target, Transaction};
use async_recursion::async_recursion;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, iter::FromIterator};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfigTypeView {
    pub name: String,
    pub widget: SensorWidgetKindView,
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
pub struct SensorConfigTypeMapId(i64);

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
    Moment,
    Map(Box<SensorWidgetKind>, Box<SensorWidgetKind>),
    PinSelection,
    Selection(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum SensorWidgetKindView {
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    String,
    Moment,
    Map(Box<SensorWidgetKindView>, Box<SensorWidgetKindView>),
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
    Moment,
    Map,
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
            SensorWidgetKind::Moment => SensorWidgetKindRaw::Moment,
            SensorWidgetKind::Map(_, _) => SensorWidgetKindRaw::Map,
            SensorWidgetKind::PinSelection => SensorWidgetKindRaw::PinSelection,
            SensorWidgetKind::Selection(_) => SensorWidgetKindRaw::Selection,
        }
    }
}

impl SensorWidgetKindView {
    pub async fn from_raw(
        txn: &mut Transaction<'_>,
        id: SensorConfigTypeId,
        raw: &SensorWidgetKindRaw,
        targets: &[&Target],
    ) -> Result<SensorWidgetKindView> {
        Self::from_raw_inner(txn, id, raw, targets, None).await
    }

    #[async_recursion]
    async fn from_raw_inner(
        txn: &mut Transaction<'_>,
        id: SensorConfigTypeId,
        raw: &SensorWidgetKindRaw,
        targets: &[&Target],
        parent: Option<(SensorConfigTypeMapId, ParentMetadata)>,
    ) -> Result<SensorWidgetKindView> {
        let kind = match raw {
            SensorWidgetKindRaw::U8 => SensorWidgetKindView::U8,
            SensorWidgetKindRaw::U16 => SensorWidgetKindView::U16,
            SensorWidgetKindRaw::U32 => SensorWidgetKindView::U32,
            SensorWidgetKindRaw::U64 => SensorWidgetKindView::U64,
            SensorWidgetKindRaw::F32 => SensorWidgetKindView::F32,
            SensorWidgetKindRaw::F64 => SensorWidgetKindView::F64,
            SensorWidgetKindRaw::String => SensorWidgetKindView::String,
            SensorWidgetKindRaw::Moment => SensorWidgetKindView::Moment,
            SensorWidgetKindRaw::Map => {
                let (map_id, key, value) = sqlx::query_as::<
                    _,
                    (
                        SensorConfigTypeMapId,
                        SensorWidgetKindRaw,
                        SensorWidgetKindRaw,
                    ),
                >(
                    "SELECT id, key, value
                     FROM sensor_config_type_selection_maps
                     WHERE type_id = $1
                          AND (parent_id = $2 OR (parent_id IS NULL AND $2 IS NULL))
                          AND (parent_metadata = $3 OR (parent_metadata IS NULL AND $3 IS NULL))",
                )
                .bind(&id)
                .bind(parent.as_ref().map(|p| p.0))
                .bind(parent.as_ref().map(|p| p.1))
                .fetch_one(&mut *txn)
                .await?;

                let key = Self::from_raw_inner(
                    &mut *txn,
                    id,
                    &key,
                    targets,
                    Some((map_id, ParentMetadata::Key)),
                )
                .await?;
                let value = Self::from_raw_inner(
                    &mut *txn,
                    id,
                    &value,
                    targets,
                    Some((map_id, ParentMetadata::Value)),
                )
                .await?;
                SensorWidgetKindView::Map(key.into(), value.into())
            }
            SensorWidgetKindRaw::Selection => {
                let options = sqlx::query_as::<_, (String,)>(
                    "SELECT option
                     FROM sensor_config_type_selection_options
                     WHERE type_id = $1
                           AND (parent_map_id = $2 OR (parent_map_id IS NULL AND $2 IS NULL))
                           AND (parent_map_metadata = $3 OR (parent_map_metadata IS NULL AND $3 IS NULL))",
                )
                    .bind(&id)
                    .bind(parent.as_ref().map(|p| p.0))
                    .bind(parent.as_ref().map(|p| p.1))
                    .fetch_all(txn)
                    .await?
                    .into_iter()
                    .map(|(opt,)| opt)
                    .collect();
                SensorWidgetKindView::Selection(options)
            }
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
                SensorWidgetKindView::Selection(pins)
            }
        };
        Ok(kind)
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

        let mut work_queue: VecDeque<(
            SensorWidgetKind,
            Option<(SensorConfigTypeMapId, ParentMetadata)>,
        )> = VecDeque::from_iter(vec![(widget, None)]);
        while let Some((widget, parent)) = work_queue.pop_front() {
            match widget {
                SensorWidgetKind::Map(key, value) => {
                    let (map_id,) = sqlx::query_as::<_, (SensorConfigTypeMapId,)>(
                        "INSERT INTO sensor_config_type_selection_maps (type_id, parent_id, parent_metadata, key, value) VALUES ($1, $2, $3, $4, $5) RETURNING id",
                    )
                        .bind(&id)
                        .bind(parent.as_ref().map(|p| p.0))
                        .bind(parent.as_ref().map(|p| p.1))
                        .bind(&SensorWidgetKindRaw::from(&*key))
                        .bind(&SensorWidgetKindRaw::from(&*value))
                        .fetch_one(&mut *txn)
                        .await?;

                    work_queue.push_back((*key, Some((map_id, ParentMetadata::Key))));
                    work_queue.push_back((*value, Some((map_id, ParentMetadata::Value))));
                }
                SensorWidgetKind::Selection(options) => {
                    for option in options {
                        sqlx::query(
                            "INSERT INTO sensor_config_type_selection_options (type_id, parent_map_id, parent_map_metadata, option) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
                        )
                            .bind(&id)
                            .bind(parent.as_ref().map(|p| p.0))
                            .bind(parent.as_ref().map(|p| p.1))
                            .bind(&option)
                            .execute(&mut *txn)
                            .await?;
                    }
                }
                _ => {}
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
    ) -> Result<SensorWidgetKindView> {
        SensorWidgetKindView::from_raw(txn, self.id, &self.widget, targets).await
    }
}

#[derive(sqlx::Type, Debug, Copy, Clone)]
enum ParentMetadata {
    Key,
    Value,
}
