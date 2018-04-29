use schema::users;

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

#[derive(Queryable, Debug, Serialize, Clone)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password_hash: String
}

#[derive(Insertable, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub email: String,
    pub password_hash: String
}
