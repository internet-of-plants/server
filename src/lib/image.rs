use std::fs::File;
use image::{load_from_memory, GenericImage, ImageError, ImageOutputFormat, Nearest};

const WIDTH: u32 = 350;
const HEIGHT: u32 = 450;

const THUMB_WIDTH: u32 = 200;
const THUMB_HEIGHT: u32 = 200;

pub fn save_image(bytes: &[u8], filename: &str) -> Result<(), ImageError> {
    let mut image = load_from_memory(bytes)?;
    if image.width() > WIDTH || image.height() > HEIGHT {
        image = image.resize(WIDTH, HEIGHT, Nearest);
    }
    let thumb = image.thumbnail(THUMB_WIDTH, THUMB_HEIGHT);

    let thumb_filename = format!("src/static/thumb/{}.jpg", filename);
    let filename = format!("src/static/images/{}.jpg", filename);

    let mut file = File::create(filename)?;
    image.write_to(&mut file, ImageOutputFormat::JPEG(120))?;

    file = File::create(thumb_filename)?;
    thumb.write_to(&mut file, ImageOutputFormat::JPEG(120))?;
    Ok(())
}
