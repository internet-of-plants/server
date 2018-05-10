use lib::utils::{Timestamp, UID};
use models::{PlantType, User};
use schema::plants;

#[macro_export]
macro_rules! PlantViewSql {
    () => ((
        plants::id,
        plants::name,
        (plant_types::all_columns),
        (users::all_columns),
        plants::timestamp
    ));
}

#[derive(Queryable, Debug, Serialize)]
pub struct PlantView {
    pub id: UID,
    pub name: String,
    pub plant_type: PlantType,
    pub user: User,
    pub timestamp: Timestamp,
}

#[derive(Queryable, Debug, Serialize, Clone)]
pub struct Plant {
    pub id: UID,
    pub name: String,
    pub type_id: UID,
    pub user_id: UID,
    pub timestamp: Timestamp,
}

#[derive(Insertable, Debug)]
#[table_name = "plants"]
pub struct NewPlant {
    pub name: String,
    pub type_id: UID,
    pub user_id: UID,
}
