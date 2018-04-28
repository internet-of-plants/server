pub type Celsius = i16;
pub type Percentage = i16;

#[derive(Queryable, Debug, Serialize)]
pub struct Event {
    pub id: i64,
    pub plant_id: i32,
    pub air_temperature: Celsius,
    pub air_humidity: Percentage,
    pub soil_temperature: Celsius,
    pub soil_resistivity: i16,
    pub light: i16,
    pub timestamp: i64
}

#[derive(Queryable, Debug, Serialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password: String
}
