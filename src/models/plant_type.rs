use lib::{schema::plant_types, utils::Timestamp, utils::UID};
use models::User;

#[derive(Serialize, Deserialize, Debug)]
pub struct PlantTypeForm {
    pub name: String,
    pub image: String,
}

#[derive(Insertable, Debug)]
#[table_name = "plant_types"]
pub struct NewPlantType {
    pub name: String,
    pub slug: String,
    pub filename: String,
    pub user_id: UID,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct PlantType {
    pub id: UID,
    pub name: String,
    pub slug: String,
    pub filename: String,
    pub user_id: UID,
    pub timestamp: Timestamp,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct PlantTypeView {
    pub id: UID,
    pub name: String,
    pub slug: String,
    pub filename: String,
    pub user: User,
    pub timestamp: Timestamp,
}

macro_rules! PlantTypeViewSql {
    () => {
        (
            plant_types::id,
            plant_types::name,
            plant_types::slug,
            plant_types::filename,
            (users::all_columns),
            plant_types::timestamp,
        )
    };
}
