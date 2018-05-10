use models::Plant;
use schema::events;
use lib::utils::{AnalogRead, BigUID, Celsius, DeviceTimestamp, Percentage, Timestamp, UID};

#[macro_export]
macro_rules! EventViewSql {
    () => ((
        events::id,
        (plants::all_columns),
        events::air_temperature_celsius,
        events::air_humidity_percentage,
        events::soil_temperature_celsius,
        events::soil_resistivity,
        events::light,
        events::device_timestamp,
        events::timestamp
    ));
}

#[derive(Queryable, Serialize, Debug)]
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

#[derive(Queryable, Serialize, Debug)]
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
