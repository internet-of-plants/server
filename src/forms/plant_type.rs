use lib::utils::{parse_multipart, MultipartDeserialize};

#[derive(Debug, Deserialize)]
pub struct PlantTypeForm {
    pub name: String,
    pub filename: String,
}

impl MultipartDeserialize for PlantTypeForm {
    fn from_multipart(content: &[u8], boundary: &[u8]) -> Option<Self> {
        let values = parse_multipart(content, boundary);
        let name = match values.get("name") {
            Some(name) => name.to_owned(),
            None => return None,
        };

        match values.get("filename") {
            Some(filename) => Some(PlantTypeForm {
                name: name,
                filename: filename.to_owned(),
            }),
            None => None,
        }
    }
}
