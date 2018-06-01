use base64::{decode, DecodeError};
use hex::ToHex;
use image::{load_from_memory, GenericImage, ImageError, ImageOutputFormat, Triangle};
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
        image = image.resize_to_fill(IMAGE_WIDTH, IMAGE_HEIGHT, Triangle);
    }
    let thumb = image.resize_to_fill(THUMB_WIDTH, THUMB_HEIGHT, Triangle);

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
        image: ::config::TEST_IMAGE.to_owned(),
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
mod tests {
    use config::{IMAGE_PATH, THUMB_PATH};
    use std::fs::remove_file;
    use std::path::Path;

    #[test]
    fn decode_b64_image() {
        let image = ::config::TEST_IMAGE;
        assert!(super::decode_b64_image(image).is_ok());
        assert!(super::decode_b64_image("").is_err());
        assert!(super::decode_b64_image("data:image/base64").is_err());
        assert!(super::decode_b64_image("base64").is_err());
        assert!(super::decode_b64_image("base64aa").is_err());
        assert!(super::decode_b64_image("a").is_err());
        assert!(super::decode_b64_image("^").is_err());
    }

    struct FileRemover<'a>(pub &'a str);
    impl<'a> Drop for FileRemover<'a> {
        fn drop(&mut self) {
            let _ = remove_file(self.0);
        }
    }

    #[test]
    fn save_image() {
        let image = super::decode_b64_image(::config::TEST_IMAGE).unwrap();

        let filename = "__test_save_image";
        let image_path = format!("{}/{}.jpg", IMAGE_PATH, filename);
        let thumb_path = format!("{}/{}.jpg", THUMB_PATH, filename);
        let (_f, _t) = (FileRemover(&image_path), FileRemover(&thumb_path));

        assert!(super::save_image(filename, &image).is_ok());
        assert!(Path::new(&image_path).exists());
        assert!(Path::new(&thumb_path).exists());

        assert!(super::save_image(filename, &[0]).is_err());
        assert!(super::save_image(filename, &[]).is_err());
    }
}
