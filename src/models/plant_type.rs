use lib::utils::UID;
use models::User;
use schema::plant_types;

#[derive(Queryable, Serialize, Debug)]
pub struct PlantTypeView {
    pub id: UID,
    pub name: String,
    pub slug: String,
    pub user: User
}

#[derive(Queryable, Serialize, Debug)]
pub struct PlantType {
    pub id: UID,
    pub name: String,
    pub slug: String,
    pub user_id: UID
}

#[derive(Insertable, Debug)]
#[table_name = "plant_types"]
pub struct NewPlantType {
    pub name: String,
    pub slug: String,
    pub user_id: UID
}
