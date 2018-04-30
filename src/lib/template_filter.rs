use tera;
use std::collections::HashMap;
use serde_json::value::{Value, to_value};

pub fn url_for_filter(value: Value, _: HashMap<String, Value>) -> Result<Value, tera::Error> {
    let url_key = try_get_value!("url_for_filter", "value",
                                 String, value);
    Ok(to_value(url_for!(&url_key)).unwrap())
}
