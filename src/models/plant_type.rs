use lib::utils::{Timestamp, UID};
use models::User;
use schema::plant_types;

#[macro_export]
macro_rules! PlantTypeViewSql {
    () => ((
        plant_types::id,
        plant_types::name,
        plant_types::slug,
        plant_types::filename,
        (users::all_columns),
        plant_types::timestamp,
    ));
}

#[derive(Queryable, Serialize, Debug)]
pub struct PlantTypeView {
    pub id: UID,
    pub name: String,
    pub slug: String,
    pub filename: String,
    pub user: User,
    pub timestamp: Timestamp,
}

#[derive(Queryable, Serialize, Debug)]
pub struct PlantType {
    pub id: UID,
    pub name: String,
    pub slug: String,
    pub filename: String,
    pub user_id: UID,
    pub timestamp: Timestamp,
}

#[derive(Insertable, Debug)]
#[table_name = "plant_types"]
pub struct NewPlantType {
    pub name: String,
    pub slug: String,
    pub filename: String,
    pub user_id: UID,
}
