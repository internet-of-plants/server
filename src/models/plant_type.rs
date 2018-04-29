use lib::utils::UID;
use schema::plant_types;

#[derive(Queryable, Debug, Serialize, Clone)]
pub struct PlantType {
    pub id: UID,
    pub name: String,
    pub slug: String,
    pub user_id: Option<UID>
}

#[derive(Insertable, Debug)]
#[table_name = "plant_types"]
pub struct NewPlantType {
    pub name: String,
    pub slug: String,
    pub user_id: UID
}
