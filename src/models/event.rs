use lib::{schema::events, utils::AnalogRead, utils::BigUID, utils::Celsius,
          utils::DeviceTimestamp, utils::Percentage, utils::Timestamp, utils::UID};
use models::Plant;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventForm {
    pub pid: UID,
    pub at: Celsius,
    pub ah: Percentage,
    pub st: Celsius,
    pub sr: AnalogRead,
    pub l: AnalogRead,
    pub t: DeviceTimestamp,
}

#[derive(Insertable, Debug)]
#[table_name = "events"]
pub struct NewEvent {
    pub plant_id: UID,
    pub air_temperature_celsius: Celsius,
    pub air_humidity_percentage: Percentage,
    pub soil_temperature_celsius: Celsius,
    pub soil_resistivity: AnalogRead,
    pub light: AnalogRead,
    pub device_timestamp: DeviceTimestamp,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct Event {
    pub id: BigUID,
    pub plant_id: UID,
    pub air_temperature: Celsius,
    pub air_humidity: Percentage,
    pub soil_temperature: Celsius,
    pub soil_resistivity: AnalogRead,
    pub light: AnalogRead,
    pub device_timestamp: DeviceTimestamp,
    pub timestamp: Timestamp,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct EventView {
    pub id: BigUID,
    pub plant: Plant,
    pub air_temperature: Celsius,
    pub air_humidity: Percentage,
    pub soil_temperature: Celsius,
    pub soil_resistivity: AnalogRead,
    pub light: AnalogRead,
    pub device_timestamp: DeviceTimestamp,
    pub timestamp: Timestamp,
}

macro_rules! EventViewSql {
    () => {
        (
            events::id,
            (plants::all_columns),
            events::air_temperature_celsius,
            events::air_humidity_percentage,
            events::soil_temperature_celsius,
            events::soil_resistivity,
            events::light,
            events::device_timestamp,
            events::timestamp,
        )
    };
}
