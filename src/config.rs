pub const HOST: &'static str = "127.0.0.1:3000";

pub const FILENAME_SIZE: usize = 20;

#[cfg(release)]
pub const LOG_PATH: &'static str = "target/log";

pub const IMAGE_WIDTH: u32 = 350;
pub const IMAGE_HEIGHT: u32 = 450;

pub const THUMB_WIDTH: u32 = 200;
pub const THUMB_HEIGHT: u32 = 200;

pub const IMAGE_PATH: &'static str = "src/static/images";
pub const THUMB_PATH: &'static str = "src/static/thumbs";

#[cfg(test)]
pub const TEST_IMAGE: &'static str = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEASABIAAD//gATQ3JlYXRlZCB3aXRoIEdJTVD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////2wBDAf//////////////////////////////////////////////////////////////////////////////////////wgARCAABAAEDAREAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAAAv/EABQBAQAAAAAAAAAAAAAAAAAAAAD/2gAMAwEAAhADEAAAASf/xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAEFAn//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAEDAQE/AX//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAECAQE/AX//xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAY/An//xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAE/IX//2gAMAwEAAgADAAAAEB//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAEDAQE/EH//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAECAQE/EH//xAAUEAEAAAAAAAAAAAAAAAAAAAAA/9oACAEBAAE/EH//2Q==";
