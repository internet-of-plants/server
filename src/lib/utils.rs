use hex::ToHex;
use sodiumoxide::crypto::hash;
use rand::RngCore;
use rand::os::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::RwLock;
use std::str::from_utf8;
use gotham::state::{FromState, State};
use serde::Deserialize;
use serde_urlencoded::from_str;
use hyper::header::ContentType;
use hyper::Headers;
use middlewares::body::{BodyData, Multipart, UrlEncoded};
use mime;

pub trait MultipartDeserialize {
    fn from_multipart(content: &[u8], boundary: &[u8]) -> Option<Self>
    where
        Self: Sized;
}

pub type UID = i32;
pub type BigUID = i64;
pub type Celsius = i16;
pub type Percentage = i16;
pub type AnalogRead = i16;

pub type DeviceTimestamp = i32;
pub type Timestamp = i64;

pub enum RawMultipartValue<'a> {
    Text(String),
    File((String, &'a [u8])),
    Invalid,
}
pub use self::RawMultipartValue::*;

type RouteTable = RwLock<HashMap<String, String>>;
lazy_static! {
    pub static ref REVERSE_ROUTE_TABLE: RouteTable = RwLock::new(HashMap::new());
}

/// Returns the number of seconds since UNIX_EPOCH (1970)
pub fn unix_epoch() -> u64 {
    // Returns 0 if now() is before UNIX_EPOCH (1970)
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_secs(),
        Err(_) => 0,
    }
}

/// Returns the hash of argument
pub fn basic_hash(src: &str) -> String {
    hash::hash(src.as_bytes()).as_ref().to_hex()
}

/// Returns randomly generated string with specified size
pub fn random_string(len: usize) -> String {
    let mut rng = OsRng::new().unwrap();
    let mut hash = String::new();
    while hash.len() < len {
        hash.push_str(&basic_hash(&rng.next_u32().to_string()));
    }

    hash[..len].to_owned()
}

fn extract_value_multipart<'a>(
    mut content: &'a [u8],
    pattern: &'a [u8],
) -> (&'a [u8], String, bool) {
    let mut value = String::new();
    if content.len() > pattern.len() && &content[..pattern.len()] == pattern {
        content = &content[pattern.len()..];
        for c in content {
            if c == &('"' as u8) {
                break;
            }
            value.push(*c as char);
        }
        content = &content[value.len() + 1..];
        (content, value, true)
    } else {
        (content, value, false)
    }
}

pub fn parse_multipart_file(content: &[u8]) -> (String, RawMultipartValue) {
    let start = "\r\nContent-Disposition: form-data; name=\"".as_bytes();
    let filename = "; filename=\"".as_bytes();
    let image_header = "Content-Type: image/".as_bytes();

    // Content-Disposition: form-data; name="<name>"
    let (content, key, _) = extract_value_multipart(content, start);

    // Content-Disposition: form-data; name="..."; filename="<filename>"
    let (mut content, value, is_file) = extract_value_multipart(content, filename);
    if !is_file {
        return (key, Invalid);
    }

    skip_newline!(content);
    // Content-Type
    if content.len() > image_header.len() && &content[..image_header.len()] == image_header {
        // Content-Type: image/
        content = &content[image_header.len()..];

        // Skip the file type: png, jpeg...
        while content.len() > 0 && content[0] > 'a' as u8 && content[0] < 'z' as u8 {
            content = &content[1..];
        }
        skip_newline!(content);
        skip_newline!(content, end);
        return (key, File((value, content)));
    } else {
        return (key, Invalid);
    }
}

pub fn parse_multipart_part(content: &[u8]) -> (String, RawMultipartValue) {
    if content == "--\r\n".as_bytes() {
        return ("invalid".to_owned(), Invalid);
    }

    let start = "\r\nContent-Disposition: form-data; name=\"".as_bytes();
    let filename = "; filename=\"".as_bytes();
    let image_header = "Content-Type: image/".as_bytes();

    // Content-Disposition: form-data; name="<name>"
    let (content, key, _) = extract_value_multipart(content, start);

    // Content-Disposition: form-data; name="..."; filename="<filename>"
    let (mut content, value, is_file) = extract_value_multipart(content, filename);
    if is_file {
        skip_newline!(content);
        // Content-Type
        if content.len() > image_header.len() && &content[..image_header.len()] == image_header {
            // Content-Type: image/
            content = &content[image_header.len()..];

            // Skip the file type: png, jpeg...
            while content.len() > 0 && content[0] > 'a' as u8 && content[0] < 'z' as u8 {
                content = &content[1..];
            }
            skip_newline!(content);
            skip_newline!(content, end);
            return (key, File((value, content)));
        } else {
            return (key, Invalid);
        }
    }

    skip_newline!(content);
    skip_newline!(content, end);

    if let Ok(content) = from_utf8(content) {
        (key, Text(content.to_owned()))
    } else {
        (key, Text("".to_owned()))
    }
}

pub fn parse_multipart(content: &[u8], boundary: &[u8]) -> HashMap<String, String> {
    let mut values: HashMap<String, String> = HashMap::new();
    let mut index = 0;
    let mut last_item = 0;

    while index < content.len() {
        if content.len() < index + boundary.len() {
            let (key, value) = parse_multipart_part(&content[last_item..]);
            match value {
                File((filename, _)) => {
                    let _ = values.insert(key, filename);
                }
                Text(value) => {
                    let _ = values.insert(key, value);
                }
                Invalid => {}
            }
            index = content.len();
        } else if &content[index..index + boundary.len()] == boundary {
            let content = &content[last_item..index];
            index += boundary.len();
            last_item = index;

            if content.len() == 0 {
                continue;
            }

            let (key, value) = parse_multipart_part(content);
            match value {
                File((filename, _)) => {
                    let _ = values.insert(key, filename);
                }
                Text(value) => {
                    let _ = values.insert(key, value);
                }
                Invalid => {}
            }
        } else {
            index += 1;
        }
    }
    values
}

pub fn parse_multipart_files<'a>(
    content: &'a [u8],
    boundary: &'a [u8],
) -> HashMap<String, &'a [u8]> {
    let mut values: HashMap<String, &'a [u8]> = HashMap::new();
    let mut index = 0;
    let mut last_item = 0;

    while index < content.len() {
        if content.len() > index + boundary.len()
            && &content[index..index + boundary.len()] == boundary
        {
            let content = &content[last_item..index];
            index += boundary.len();
            last_item = index;

            if content.len() == 0 {
                continue;
            }

            let (key, file) = parse_multipart_file(content);
            match file {
                File((_, file)) => {
                    let _ = values.insert(key, file);
                }
                _ => {}
            }
        } else {
            index += 1;
        }
    }
    values
}

pub fn from_body<'a, T: MultipartDeserialize + Deserialize<'a>>(state: &'a mut State) -> Option<T> {
    let boundary = multipart_boundary(state);

    match (BodyData::try_borrow_from(state), boundary) {
        (Some(&BodyData(UrlEncoded(ref raw))), None) => match from_str::<T>(raw) {
            Ok(value) => Some(value),
            Err(_) => None,
        },
        (Some(&BodyData(Multipart(ref raw))), Some(ref boundary)) => {
            T::from_multipart(raw, boundary.as_bytes())
        }
        _ => None,
    }
}

pub fn multipart_boundary(state: &State) -> Option<String> {
    if let Some(content_type) = Headers::borrow_from(&state).get::<ContentType>() {
        let content_type = content_type.as_ref();
        let form_data = format!("{}; boundary=", mime::MULTIPART_FORM_DATA.as_ref());
        if content_type.len() > form_data.len() && &content_type[..form_data.len()] == form_data {
            Some(format!("--{}", &content_type[form_data.len()..]))
        } else {
            None
        }
    } else {
        None
    }
}
