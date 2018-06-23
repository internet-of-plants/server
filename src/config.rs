use lib::utils::random_string;
use std::env::var;

lazy_static! {
    pub static ref SECRET_KEY: [u8; 32] = {
        let mut key: [u8; 32] = Default::default();
        let string = if let Ok(ref secret_key) = var("SECRET_KEY") {
            secret_key.clone()
        } else {
            random_string(32).unwrap()
        };
        key.copy_from_slice(&string.as_bytes()[..32]);
        key
    };
    pub static ref HOST: String = match var("PORT") {
        Ok(ref port) => format!("0.0.0.0:{}", port),
        _ => "0.0.0.0:3000".to_owned(),
    };
}

#[cfg(release)]
pub const LOG_PATH: &'static str = "target/log";

// Bytes
pub const REQUEST_SIZE_LIMIT: usize = 5_000_000;

pub const FILENAME_SIZE: usize = 20;

pub const IMAGE_WIDTH: u32 = 350;
pub const IMAGE_HEIGHT: u32 = 450;

pub const THUMB_WIDTH: u32 = 200;
pub const THUMB_HEIGHT: u32 = 200;

pub const STATIC_PATH: &'static str = "src/static/";
pub const IMAGE_PATH: &'static str = "src/static/images";
pub const THUMB_PATH: &'static str = "src/static/thumbs";

#[cfg(test)]
pub const TEST_IMAGE: &'static str = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEASABIAAD//gATQ3JlYXRlZCB3aXRoIEdJTVD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////2wBDAf//////////////////////////////////////////////////////////////////////////////////////wgARCAABAAEDAREAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAAAv/EABQBAQAAAAAAAAAAAAAAAAAAAAD/2gAMAwEAAhADEAAAASf/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAEFAn//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAEDAQE/AX//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAECAQE/AX//xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAY/An//xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAE/IX//2gAMAwEAAgADAAAAEB//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAEDAQE/EH//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAECAQE/EH//xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAE/EH//2Q==";
