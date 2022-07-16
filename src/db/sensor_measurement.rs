use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SensorMeasurementKind {
    AirTemperature,
    SoilTemperature,
    AirHumidity,
    SoilMoisture,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SensorMeasurementType {
    FloatCelsius,
    Percentage,
    RawAnalogRead, // (0-1024)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SensorMeasurementView {
    pub name: String,
    pub human_name: String,
    pub ty: SensorMeasurementType,
    pub kind: SensorMeasurementKind,
    pub color: String,
}

impl SensorMeasurementView {
    pub fn new(m: SensorMeasurement, name: String, color: String) -> Self {
        Self {
            human_name: m.human_name,
            name,
            ty: m.ty,
            kind: m.kind,
            color,
        }
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorMeasurement {
    pub human_name: String,
    pub name: String,
    pub value: String,
    pub ty: SensorMeasurementType,
    pub kind: SensorMeasurementKind,
}
