use derive_get::Getters;
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

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SensorMeasurementView {
    variable_name: String,
    name: String,
    ty: SensorMeasurementType,
    kind: SensorMeasurementKind,
    color: String,
}

impl SensorMeasurementView {
    pub fn new(m: SensorMeasurement, variable_name: String, color: String) -> Self {
        Self {
            name: m.name,
            variable_name,
            ty: m.ty,
            kind: m.kind,
            color,
        }
    }
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorMeasurement {
    pub name: String,
    pub variable_name: String,
    pub value: String,
    pub ty: SensorMeasurementType,
    pub kind: SensorMeasurementKind,
}
