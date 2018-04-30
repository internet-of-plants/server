use models::Plant;
use lib::utils::{Celsius, Percentage, UID, BigUID, AnalogRead};

#[derive(Queryable, Serialize, Debug)]
pub struct EventView {
    pub id: BigUID,
    pub plant: Plant,
    pub air_temperature: Celsius,
    pub air_humidity: Percentage,
    pub soil_temperature: Celsius,
    pub soil_resistivity: AnalogRead,
    pub light: AnalogRead,
    pub timestamp: i64
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
    pub timestamp: i64
}
