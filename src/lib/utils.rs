use base64::{decode, DecodeError};
use hex::ToHex;
use image::{load_from_memory, GenericImage, ImageError, ImageOutputFormat, Nearest};
use rand::{RngCore, os::OsRng};
use sodiumoxide::crypto::hash;
use std::fs::File;

use config::{IMAGE_HEIGHT, IMAGE_PATH, IMAGE_WIDTH, THUMB_HEIGHT, THUMB_PATH, THUMB_WIDTH};
use lib::error::Error;

pub type UID = i32;
pub type BigUID = i64;
pub type Celsius = i16;
pub type Percentage = i16;
pub type AnalogRead = i16;

pub type DeviceTimestamp = i32;
pub type Timestamp = i64;

/// Resize image according to config, create thumb, save them
pub fn save_image(filename: &str, bytes: &[u8]) -> Result<(), ImageError> {
    let mut image = load_from_memory(bytes)?;

    if image.width() > IMAGE_WIDTH || image.height() > IMAGE_HEIGHT {
        image = image.resize(IMAGE_WIDTH, IMAGE_HEIGHT, Nearest);
    }
    let thumb = image.thumbnail(THUMB_WIDTH, THUMB_HEIGHT);

    let thumb_filename = format!("{}/{}.jpg", THUMB_PATH, filename);
    let filename = format!("{}/{}.jpg", IMAGE_PATH, filename);

    let mut file = File::create(filename)?;
    image.write_to(&mut file, ImageOutputFormat::JPEG(120))?;

    file = File::create(thumb_filename)?;
    thumb.write_to(&mut file, ImageOutputFormat::JPEG(120))?;
    Ok(())
}

/// Returns the hash of argument
pub fn basic_hash(src: &str) -> String {
    hash::hash(src.as_bytes()).as_ref().to_hex()
}

/// Returns randomly generated string with specified size
pub fn random_string(len: usize) -> Result<String, Error> {
    let mut rng = OsRng::new()?;
    let mut hash = String::new();
    while hash.len() < len {
        hash.push_str(&basic_hash(&rng.next_u32().to_string()));
    }

    Ok(hash[..len].to_owned())
}

/// Decode base64 image data from browsers to binary
pub fn decode_b64_image(image: &str) -> Result<Vec<u8>, Error> {
    let find = "base64";
    let start = "data:image/";
    let (_, data) = match (image.find(find), image.find(start)) {
        (Some(index), Some(_)) if index + find.len() + 1 < image.len() => {
            image.split_at(index + find.len() + 1)
        }
        _ => return Err(Error::Base64(DecodeError::InvalidByte(0, 0))),
    };
    Ok(decode(data)?)
}

#[cfg(test)]
/// Get cookie String from TestRequest
pub fn extract_cookie(r: &::actix_web::client::ClientResponse) -> String {
    use actix_web::{HttpMessage, http::header::HeaderValue};
    r.headers()
        .get("set-cookie")
        .unwrap_or(&HeaderValue::from_str("s=").unwrap())
        .to_str()
        .unwrap()
        .to_owned()
}

#[cfg(test)]
/// Authenticate user (create if not existant) and return cookie String
pub fn authenticate_tester(srv: &mut ::actix_web::test::TestServer) -> String {
    use actix_web::http::{Method, StatusCode};
    use models::{SigninForm, SignupForm};
    let body = SigninForm {
        login: "tester".to_owned(),
        password: "password".to_owned(),
    };
    let req = srv.client(Method::POST, "/signin").json(body).unwrap();
    let mut r = srv.execute(req.send()).unwrap();

    if r.status() == StatusCode::UNAUTHORIZED {
        let body = SignupForm {
            username: "tester".to_owned(),
            email: "tester@example.com".to_owned(),
            password: "password".to_owned(),
        };
        let req = srv.client(Method::POST, "/signup").json(body).unwrap();
        r = srv.execute(req.send()).unwrap();
    }

    extract_cookie(&r)
}

#[cfg(test)]
/// Create plant type and return its id
pub fn create_plant_type(srv: &mut ::actix_web::test::TestServer, cookie: &str) -> i32 {
    use actix_web::HttpMessage;
    use actix_web::http::{Cookie, Method};
    use futures::future::Future;
    use models::{PlantType, PlantTypeForm};
    let body = PlantTypeForm {
        name: "plant_typer".to_owned(),
        image: "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=".to_owned(),
    };
    let req = srv.client(Method::POST, "/plant_type")
        .cookie(Cookie::parse(cookie).unwrap())
        .json(body)
        .unwrap();
    let r = srv.execute(req.send()).unwrap();
    r.json::<PlantType>().wait().unwrap().id
}

#[cfg(test)]
/// Create plant and return its id
pub fn create_plant(srv: &mut ::actix_web::test::TestServer, cookie: &str) -> i32 {
    use actix_web::HttpMessage;
    use actix_web::http::{Cookie, Method};
    use futures::future::Future;
    use models::{Plant, PlantForm};
    let plant_type_id = create_plant_type(srv, &cookie);
    let body = PlantForm {
        name: "planter".to_owned(),
        type_id: plant_type_id,
    };
    let req = srv.client(Method::POST, "/plant")
        .cookie(Cookie::parse(cookie).unwrap())
        .json(body)
        .unwrap();
    let r = srv.execute(req.send()).unwrap();
    r.json::<Plant>().wait().unwrap().id
}

#[cfg(test)]
pub fn clean_db() {
    use diesel::{self, RunQueryDsl};
    use lib::db::{connection, pool};
    use lib::schema::*;

    let p = pool();
    diesel::delete(events::table)
        .execute(&*connection(&p).unwrap())
        .unwrap();
    diesel::delete(plants::table)
        .execute(&*connection(&p).unwrap())
        .unwrap();
    diesel::delete(plant_types::table)
        .execute(&*connection(&p).unwrap())
        .unwrap();
    diesel::delete(users::table)
        .execute(&*connection(&p).unwrap())
        .unwrap();
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn decode_b64_image() {
        let image = "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=";
        assert!(super::decode_b64_image(image).is_ok());
        assert!(super::decode_b64_image("").is_err());
        assert!(super::decode_b64_image("data:image/base64").is_err());
        assert!(super::decode_b64_image("base64").is_err());
        assert!(super::decode_b64_image("base64aa").is_err());
        assert!(super::decode_b64_image("a").is_err());
        assert!(super::decode_b64_image("^").is_err());
    }

    #[test]
    fn save_image() {
        let image = super::decode_b64_image(
            "data:image/gif;base64,R0lGODlhAQABAAD/ACwAAAAAAQABAAACADs=",
        ).unwrap();

        let filename = "__test_save_image";
        assert!(super::save_image(filename, &image).is_ok());
        assert!(Path::new(&format!("image/{}.jpg", filename)).exists());
        assert!(Path::new(&format!("thumb/{}.jpg", filename)).exists());

        assert!(super::save_image(filename, &[0]).is_err());
        assert!(super::save_image(filename, &[]).is_err());
    }
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
