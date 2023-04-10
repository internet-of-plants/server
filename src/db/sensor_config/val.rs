use crate::{Error, Result, Sensor, SensorId, SensorWidgetKindView, Transaction};
use async_recursion::async_recursion;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ElementRaw {
    key: ValRaw,
    value: ValRaw,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ValRaw {
    // Includes pins
    String(String),
    // Includes Sensor widget
    Integer(u64),
    Float(f64),
    Moment { hours: u8, minutes: u8, seconds: u8 },
    Map(Vec<ElementRaw>),
}

#[derive(Getters, Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Element {
    key: Val,
    value: Val,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum Val {
    Symbol(String),
    Integer(u64),
    Float(f64),
    #[serde(rename_all = "camelCase")]
    Sensor {
        sensor_id: SensorId,
    },
    Moment {
        hours: u8,
        minutes: u8,
        seconds: u8,
    },
    Map(Vec<Element>),
}

impl From<Val> for ValRaw {
    // FIXME: recursion
    fn from(val: Val) -> Self {
        match val {
            Val::Symbol(string) => ValRaw::String(string),
            Val::Integer(num) => ValRaw::Integer(num),
            Val::Float(num) => ValRaw::Float(num),
            Val::Sensor { sensor_id } => ValRaw::Integer(i64::from(sensor_id) as u64),
            Val::Moment {
                hours,
                minutes,
                seconds,
            } => ValRaw::Moment {
                hours,
                minutes,
                seconds,
            },
            Val::Map(els) => {
                let mut raws = Vec::with_capacity(els.len());
                for el in els {
                    raws.push(ElementRaw {
                        key: el.key.into(),
                        value: el.value.into(),
                    });
                }
                ValRaw::Map(raws)
            }
        }
    }
}

impl Val {
    #[async_recursion]
    pub async fn new(
        txn: &mut Transaction<'_>,
        raw: ValRaw,
        widget: SensorWidgetKindView,
    ) -> Result<Self> {
        match &raw {
            raw @ ValRaw::String(symbol) => match &widget {
                SensorWidgetKindView::Selection(selection) => {
                    if !selection.contains(symbol) {
                        Err(Error::InvalidSelection(symbol.clone(), selection.clone()))
                    } else {
                        Ok(Val::Symbol(symbol.clone()))
                    }
                }
                SensorWidgetKindView::U8 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::U16 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::U32 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::U64 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::F32 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::F64 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Seconds => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Sensor(_) => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Moment => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Map(_, _) => Err(Error::InvalidValType(raw.clone(), widget)),
            },
            raw @ ValRaw::Float(number) => match &widget {
                SensorWidgetKindView::F32 => Ok(Val::Float((*number as f32).into())),
                SensorWidgetKindView::F64 => Ok(Val::Float(*number)),
                SensorWidgetKindView::U8 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::U16 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::U32 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::U64 => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Seconds => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Sensor(_) => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Moment => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Map(_, _) => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Selection(_) => {
                    Err(Error::InvalidValType(raw.clone(), widget))
                }
            },
            raw @ ValRaw::Integer(number) => match &widget {
                SensorWidgetKindView::Sensor(sensor_prototype_id) => {
                    if *number >= (i64::max_value() as u64) {
                        return Err(Error::IntegerOutOfRange(*number, widget));
                    }

                    let sensor_id = SensorId::from(*number as i64);
                    let sensor = Sensor::raw_find_by_id(txn, sensor_id).await?;
                    if sensor.prototype_id() != *sensor_prototype_id {
                        Err(Error::WrongSensorKind(
                            sensor.prototype_id(),
                            *sensor_prototype_id,
                        ))
                    } else {
                        let prototype = sensor.prototype(txn).await?;
                        if prototype.variable_name().is_none() {
                            Err(Error::NoVariableNameForReferencedSensor(prototype.id()))
                        } else {
                            Ok(Val::Sensor { sensor_id })
                        }
                    }
                }
                SensorWidgetKindView::Seconds => {
                    if *number >= 60 {
                        Err(Error::IntegerOutOfRange(*number, widget))
                    } else {
                        Ok(Val::Integer(*number))
                    }
                }
                SensorWidgetKindView::U8 => {
                    if *number > (u8::max_value() as u64) {
                        Err(Error::IntegerOutOfRange(*number, widget))
                    } else {
                        Ok(Val::Integer(*number))
                    }
                }
                SensorWidgetKindView::U16 => {
                    if *number > (u16::max_value() as u64) {
                        Err(Error::IntegerOutOfRange(*number, widget))
                    } else {
                        Ok(Val::Integer(*number))
                    }
                }
                SensorWidgetKindView::U32 => {
                    if *number > (u32::max_value() as u64) {
                        Err(Error::IntegerOutOfRange(*number, widget))
                    } else {
                        Ok(Val::Integer(*number))
                    }
                }
                SensorWidgetKindView::U64 => Ok(Val::Integer(*number)),
                SensorWidgetKindView::F32 => Ok(Val::Float((*number as f32).into())),
                SensorWidgetKindView::F64 => Ok(Val::Float(*number as f64)),
                SensorWidgetKindView::Moment => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Map(_, _) => Err(Error::InvalidValType(raw.clone(), widget)),
                SensorWidgetKindView::Selection(_) => {
                    Err(Error::InvalidValType(raw.clone(), widget))
                }
            },
            raw @ ValRaw::Moment {
                hours,
                minutes,
                seconds,
            } => {
                if let SensorWidgetKindView::Moment = widget {
                    if *hours >= 24 || *minutes >= 60 || *seconds >= 60 {
                        Err(Error::InvalidMoment(*hours, *minutes, *seconds))
                    } else {
                        Ok(Val::Moment {
                            hours: *hours,
                            minutes: *minutes,
                            seconds: *seconds,
                        })
                    }
                } else {
                    Err(Error::InvalidValType(raw.clone(), widget))
                }
            }
            raw @ ValRaw::Map(raw_elements) => {
                if let SensorWidgetKindView::Map(key, value) = widget {
                    let mut elements = Vec::with_capacity(raw_elements.len());
                    for raw in raw_elements {
                        let key = Self::new(txn, raw.key.clone(), *key.clone()).await?;
                        let value = Self::new(txn, raw.value.clone(), *value.clone()).await?;
                        elements.push(Element { key, value });
                    }
                    Ok(Val::Map(elements))
                } else {
                    Err(Error::InvalidValType(raw.clone(), widget))
                }
            }
        }
    }

    #[async_recursion]
    pub async fn compile(
        &self,
        txn: &mut Transaction<'_>,
        widget: SensorWidgetKindView,
    ) -> Result<String> {
        match self {
            Val::Integer(number) => Ok(number.to_string()),
            Val::Float(number) => Ok(number.to_string()),
            Val::Symbol(string) => Ok(string.clone()),
            Val::Sensor { sensor_id } => {
                let sensor = Sensor::raw_find_by_id(txn, *sensor_id).await?;
                let prototype = sensor.prototype(txn).await?;
                let variable_name = prototype
                    .variable_name()
                    .as_ref()
                    .ok_or_else(|| Error::NoVariableNameForReferencedSensor(prototype.id()))?;

                Ok(format!("{variable_name}{}", sensor.index()))
            }
            Val::Moment {
                hours,
                minutes,
                seconds,
            } => Ok(format!("relay::Moment({hours}, {minutes}, {seconds})")),
            val @ Val::Map(vec) => {
                let mut string = String::new();
                string.push('{');
                let (key, value) = if let SensorWidgetKindView::Map(key, value) = &widget {
                    (key.clone(), value.clone())
                } else {
                    return Err(Error::InvalidValType(val.clone().into(), widget));
                };

                for (index, el) in vec.iter().enumerate() {
                    let separator = if index + 1 == vec.len() { "" } else { "," };
                    string.push_str(&format!(
                        "\n  std::make_pair({}, {}){separator}",
                        el.key.compile(txn, *key.clone()).await?,
                        el.value.compile(txn, *value.clone()).await?,
                    ));
                }

                string.push_str("\n}");

                Ok(string)
            }
        }
    }
}
