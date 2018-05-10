use tera;
use std::collections::HashMap;
use serde_json::value::{from_value, to_value, Value};

pub fn extension_filter(value: Value, _: HashMap<String, Value>) -> Result<Value, tera::Error> {
    let filename = try_get_value!("extension_filter", "value", String, value);
    Ok(to_value(format!("{}.jpg", filename)).unwrap())
}

pub fn str_filter(value: Value, _: HashMap<String, Value>) -> Result<Value, tera::Error> {
    match from_value::<i64>(value) {
        Ok(v) => Ok(to_value(&format!("{}", v)).unwrap()),
        Err(_) => Ok(to_value("").unwrap()),
    }
}

pub fn url_for_filter(value: Value, args: HashMap<String, Value>) -> Result<Value, tera::Error> {
    let url_key = try_get_value!("url_for_filter", "value", String, value);

    let mut map: HashMap<String, String> = HashMap::new();
    for (key, value) in args {
        match from_value::<String>(value) {
            Ok(v) => map.insert(key, v),
            Err(_) => None,
        };
    }
    Ok(to_value(url_for!(&url_key, map)).unwrap())
}
