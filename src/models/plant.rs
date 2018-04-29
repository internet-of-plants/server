use lib::utils::UID;
use schema::plants;

#[derive(Queryable, Debug, Serialize, Clone)]
pub struct Plant {
    pub id: UID,
    pub name: String,
    pub type_slug: String,
    pub user_id: UID
}

#[derive(Insertable, Debug)]
#[table_name = "plants"]
pub struct NewPlant {
    pub name: String,
    pub type_slug: String,
    pub user_id: UID
}
