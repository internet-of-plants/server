use crate::{Result, SensorPrototypeId, Target, Transaction};
use async_recursion::async_recursion;
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, iter::FromIterator};

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfigTypeView {
    name: Option<String>,
    widget: SensorWidgetKindView,
}

impl SensorConfigTypeView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        ty: SensorConfigType,
        targets: &[&Target],
    ) -> Result<Self> {
        Ok(Self {
            name: ty.name().clone(),
            widget: ty.widget(txn, targets).await?,
        })
    }
}

#[id]
pub struct SensorConfigTypeMapId;

#[id]
pub struct SensorConfigTypeId;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum NewSensorWidgetKind {
    Seconds,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Sensor(String),
    Moment,
    Map(Box<NewSensorWidgetKind>, Box<NewSensorWidgetKind>),
    PinSelection,
    Selection(Vec<String>),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", content = "data")]
pub enum SensorWidgetKindView {
    Seconds,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Sensor(SensorPrototypeId),
    Moment,
    Map(Box<SensorWidgetKindView>, Box<SensorWidgetKindView>),
    Selection(Vec<String>),
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SensorWidgetKindRaw {
    Seconds,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Sensor,
    Moment,
    Map,
    PinSelection,
    Selection,
}

impl From<&NewSensorWidgetKind> for SensorWidgetKindRaw {
    fn from(kind: &NewSensorWidgetKind) -> Self {
        match kind {
            NewSensorWidgetKind::Seconds => SensorWidgetKindRaw::Seconds,
            NewSensorWidgetKind::U8 => SensorWidgetKindRaw::U8,
            NewSensorWidgetKind::U16 => SensorWidgetKindRaw::U16,
            NewSensorWidgetKind::U32 => SensorWidgetKindRaw::U32,
            NewSensorWidgetKind::U64 => SensorWidgetKindRaw::U64,
            NewSensorWidgetKind::F32 => SensorWidgetKindRaw::F32,
            NewSensorWidgetKind::F64 => SensorWidgetKindRaw::F64,
            NewSensorWidgetKind::Sensor(_) => SensorWidgetKindRaw::Sensor,
            NewSensorWidgetKind::Moment => SensorWidgetKindRaw::Moment,
            NewSensorWidgetKind::Map(_, _) => SensorWidgetKindRaw::Map,
            NewSensorWidgetKind::PinSelection => SensorWidgetKindRaw::PinSelection,
            NewSensorWidgetKind::Selection(_) => SensorWidgetKindRaw::Selection,
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
            SensorWidgetKindRaw::Seconds => SensorWidgetKindView::Seconds,
            SensorWidgetKindRaw::U8 => SensorWidgetKindView::U8,
            SensorWidgetKindRaw::U16 => SensorWidgetKindView::U16,
            SensorWidgetKindRaw::U32 => SensorWidgetKindView::U32,
            SensorWidgetKindRaw::U64 => SensorWidgetKindView::U64,
            SensorWidgetKindRaw::F32 => SensorWidgetKindView::F32,
            SensorWidgetKindRaw::F64 => SensorWidgetKindView::F64,
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
                .bind(id)
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
            SensorWidgetKindRaw::Sensor => {
                let (sensor_prototype_id,) = sqlx::query_as::<_, (SensorPrototypeId,)>(
                    "SELECT sensor_prototypes.id
                     FROM sensor_config_type_selection_sensors
                     INNER JOIN sensor_prototypes ON sensor_prototypes.name = sensor_prototype_name
                     WHERE type_id = $1
                           AND (parent_id = $2 OR (parent_id IS NULL AND $2 IS NULL))
                           AND (parent_metadata = $3 OR (parent_metadata IS NULL AND $3 IS NULL))",
                )
                .bind(id)
                .bind(parent.as_ref().map(|p| p.0))
                .bind(parent.as_ref().map(|p| p.1))
                .fetch_one(txn)
                .await?;
                SensorWidgetKindView::Sensor(sensor_prototype_id)
            }
            SensorWidgetKindRaw::Selection => {
                let options = sqlx::query_as::<_, (String,)>(
                    "SELECT option
                     FROM sensor_config_type_selection_options
                     WHERE type_id = $1
                           AND (parent_id = $2 OR (parent_id IS NULL AND $2 IS NULL))
                           AND (parent_metadata = $3 OR (parent_metadata IS NULL AND $3 IS NULL))",
                )
                .bind(id)
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

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorConfigType {
    #[copy]
    id: SensorConfigTypeId,
    name: Option<String>,
    #[skip]
    widget: SensorWidgetKindRaw,
}

impl SensorConfigType {
    pub async fn find_by_id(txn: &mut Transaction<'_>, id: SensorConfigTypeId) -> Result<Self> {
        let ty = sqlx::query_as("SELECT id, name, widget FROM sensor_config_types WHERE id = $1")
            .bind(id)
            .fetch_one(txn)
            .await?;
        Ok(ty)
    }

    // TODO: find_or_create
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: Option<String>,
        widget: NewSensorWidgetKind,
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
            NewSensorWidgetKind,
            Option<(SensorConfigTypeMapId, ParentMetadata)>,
        )> = VecDeque::from_iter(vec![(widget, None)]);
        while let Some((widget, parent)) = work_queue.pop_front() {
            match widget {
                NewSensorWidgetKind::Map(key, value) => {
                    let (map_id,) = sqlx::query_as::<_, (SensorConfigTypeMapId,)>(
                        "INSERT INTO sensor_config_type_selection_maps (type_id, parent_id, parent_metadata, key, value) VALUES ($1, $2, $3, $4, $5) RETURNING id",
                    )
                        .bind(id)
                        .bind(parent.as_ref().map(|p| p.0))
                        .bind(parent.as_ref().map(|p| p.1))
                        .bind(&SensorWidgetKindRaw::from(&*key))
                        .bind(&SensorWidgetKindRaw::from(&*value))
                        .fetch_one(&mut *txn)
                        .await?;

                    work_queue.push_back((*key, Some((map_id, ParentMetadata::Key))));
                    work_queue.push_back((*value, Some((map_id, ParentMetadata::Value))));
                }
                NewSensorWidgetKind::Selection(options) => {
                    for option in options {
                        sqlx::query(
                            "INSERT INTO sensor_config_type_selection_options (type_id, parent_id, parent_metadata, option) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
                        )
                            .bind(id)
                            .bind(parent.as_ref().map(|p| p.0))
                            .bind(parent.as_ref().map(|p| p.1))
                            .bind(&option)
                            .execute(&mut *txn)
                            .await?;
                    }
                }
                NewSensorWidgetKind::Sensor(sensor_prototype_name) => {
                    sqlx::query(
                        "INSERT INTO sensor_config_type_selection_sensors (type_id, parent_id, parent_metadata, sensor_prototype_name) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
                    )
                        .bind(id)
                        .bind(parent.as_ref().map(|p| p.0))
                        .bind(parent.as_ref().map(|p| p.1))
                        .bind(&sensor_prototype_name)
                        .execute(&mut *txn)
                        .await?;
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
