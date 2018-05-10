use lib::utils::{parse_multipart, MultipartDeserialize, UID};

#[derive(Debug, Deserialize)]
pub struct PlantForm {
    pub name: String,
    pub type_id: UID,
}

impl MultipartDeserialize for PlantForm {
    fn from_multipart(content: &[u8], boundary: &[u8]) -> Option<Self> {
        let values = parse_multipart(content, boundary);
        let name = match values.get("name") {
            Some(name) => name.to_owned(),
            None => return None,
        };

        match values.get("type_id") {
            Some(type_id) => match UID::from_str_radix(type_id, 10) {
                Ok(type_id) => Some(PlantForm {
                    name: name,
                    type_id: type_id,
                }),
                Err(_) => None,
            },
            None => None,
        }
    }
}
