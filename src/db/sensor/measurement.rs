use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MeasurementKind {
    AirTemperature,
    SoilTemperature,
    AirHumidity,
    SoilMoisture,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MeasurementType {
    FloatCelsius,
    Percentage,
    RawAnalogRead, // (0-1024)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MeasurementView {
    pub name: String,
    pub human_name: String,
    pub ty: MeasurementType,
    pub kind: MeasurementKind,
    pub color: String
}

impl MeasurementView {
    pub fn new(m: Measurement, name: String, color: String) -> Self {
        Self {
            human_name: m.human_name,
            name,
            ty: m.ty,
            kind: m.kind,
            color
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Measurement {
    pub human_name: String,
    pub name: String,
    pub value: String,
    pub ty: MeasurementType,
    pub kind: MeasurementKind,
}
