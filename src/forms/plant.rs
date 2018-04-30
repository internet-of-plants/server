use lib::utils::UID;

#[derive(Debug, Deserialize)]
pub struct PlantForm {
    pub name: String,
    pub type_id: UID,
}
