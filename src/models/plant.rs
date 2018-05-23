use lib::{schema::plants, utils::BigUID, utils::Timestamp, utils::UID};
use models::{Event, PlantType, UserView};

#[derive(Serialize, Deserialize, Debug)]
pub struct PlantForm {
    pub name: String,
    pub type_id: UID,
}

#[derive(Insertable, Debug)]
#[table_name = "plants"]
pub struct NewPlant {
    pub name: String,
    pub type_id: UID,
    pub user_id: UID,
}

#[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
pub struct Plant {
    pub id: UID,
    pub name: String,
    pub type_id: UID,
    pub user_id: UID,
    pub last_event_id: Option<BigUID>,
    pub timestamp: Timestamp,
}

#[derive(Queryable, Serialize, Deserialize, Debug)]
pub struct PlantView {
    pub id: UID,
    pub name: String,
    pub plant_type: PlantType,
    pub user: UserView,
    pub last_event: Option<Event>,
    pub timestamp: Timestamp,
}

macro_rules! PlantViewSql {
    () => {
        (
            plants::id,
            plants::name,
            (plant_types::all_columns),
            (users::all_columns),
            (events::all_columns.nullable()),
            plants::timestamp,
        )
    };
}
