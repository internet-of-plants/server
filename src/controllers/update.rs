use crate::prelude::*;
use bytes::BufMut;
use controllers::Result;
use futures::TryStreamExt;
use std::fmt::Write;
use warp::filters::multipart::{FormData, Part};

pub async fn new(
    plant_id: i64,
    pool: &'static Pool,
    user_id: i64,
    form: FormData,
) -> Result<impl Reply> {
    let parts: Vec<Part> = form.try_collect().await.map_err(Error::Warp)?;
    let part = match parts.into_iter().next() {
        Some(part) => part,
        None => {
            warn!("Expected one file");
            return Err(Error::BadData.into());
        }
    };

    // Creates binary folders if non-existent
    // If user lacks permission to create folder the binary save will fail too (this would be critical)
    let _ = tokio::fs::create_dir("bins").await;

    let binary = part
        .stream()
        .try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        })
        .await
        .map_err(Error::from)?;
    let md5 = md5::compute(&binary);
    let md5 = &*md5;
    let mut file_hash = String::with_capacity(md5.len() * 2);
    for byte in md5 {
        write!(file_hash, "{:02X}", byte).map_err(Error::from)?;
    }

    let now = api::now(pool).await?;
    let filename = format!("bins/{}-{}-{}.bin", user_id, now, file_hash);
    let mut file = tokio::fs::File::create(&filename)
        .await
        .map_err(Error::from)?;
    // TODO: we should just use a stream to save the file
    // TODO: fs + db is not atomic, shutdowns or panics can mess things up, we need a task to clean leaked binaries
    // TODO: using an external file has problems, detaching it from the db, is it the right choice?
    file.write_all(&binary).await.map_err(Error::from)?;

    // TODO: storing a path in the db like this to read without care allows a hacker of the DB to hijack the server box too
    if let Err(err) =
        api::update::new(pool, user_id, plant_id, file_hash, filename.clone()).await
    {
        // Best effort
        tokio::fs::remove_file(filename)
            .await
            .map_err(Error::from)?;
        return Err(err.into());
    }
    Ok(StatusCode::OK)
}

pub async fn get(pool: &'static Pool, user_id: i64, headers: warp::http::HeaderMap) -> Result<impl Reply> {
    //let chip_ip = headers.get("x-ESP8266-Chip-ID");
    let mac_address = headers.get("x-ESP8266-STA-MAC").ok_or(Error::Forbidden)?.to_str().map_err(|_|Error::Forbidden)?;
    //let ap_mac = headers.get("x-ESP8266-AP-MAC");
    //let free_space = headers.get("x-ESP8266-free-space");
    //let sketch_size = headers.get("x-ESP8266-sketch-size");
    let md5 = headers.get("x-ESP8266-sketch-md5").ok_or(Error::Forbidden)?.to_str().map_err(|_|Error::Forbidden)?;
    //let chip_size = headers.get("x-ESP8266-chip-size");
    //let sdk_version = headers.get("x-ESP8266-sdk-version");

    let plant_id = api::plant::put(pool, user_id, mac_address.to_owned()).await?;
    let update = api::update::get(pool, user_id, plant_id).await?;
    if update.file_hash == md5 {
        return Err(Error::NothingFound.into());
    }

    let content = tokio::fs::read(update.file_name).await.map_err(Error::from)?;
    let md5 = md5::compute(&content);
    let md5 = &*md5;
    let mut file_hash = String::with_capacity(md5.len() * 2);
    for byte in md5 {
        write!(file_hash, "{:02X}", byte).map_err(Error::from)?;
    }
    if file_hash != update.file_hash {
        error!("Binary md5 didn't match the expected: {} != {}", file_hash, update.file_hash);
        return Err(Error::CorruptBinary)?;
    }
    Ok(http::Response::builder()
        .header("Content-Type", "application/octet-stream")
        .header("Content-Length", content.len().to_string())
        .header("Content-Disposition", format!("attachment; filename={}.bin", file_hash))
        .header("x-MD5", file_hash)
        .body(hyper::Body::from(content)))
}
