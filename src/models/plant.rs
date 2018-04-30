use lib::utils::UID;
use models::{User, PlantType};
use schema::plants;

#[derive(Queryable, Debug, Serialize)]
pub struct PlantView {
    pub id: UID,
    pub name: String,
    pub plant_type: PlantType,
    pub user: User
}

#[derive(Queryable, Debug, Serialize, Clone)]
pub struct Plant {
    pub id: UID,
    pub name: String,
    pub type_id: UID,
    pub user_id: UID
}

#[derive(Insertable, Debug)]
#[table_name = "plants"]
pub struct NewPlant {
    pub name: String,
    pub type_id: UID,
    pub user_id: UID
}
