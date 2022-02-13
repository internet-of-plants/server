use crate::DeviceId;
use crate::prelude::*;
use bytes::{Buf, BufMut};
use controllers::Result;
use futures::{StreamExt, TryStreamExt};
use std::fmt::Write;
use warp::filters::multipart::FormData;

pub async fn new(
    device_id: DeviceId,
    pool: &'static Pool,
    auth: Auth,
    mut form: FormData,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    //db::plant::owns(&mut txn, auth.user_id, device_id).await?;

    let mut version = form
        .next()
        .await
        .ok_or(Error::BadData)?
        .map_err(Error::Warp)?;
    let file = form
        .next()
        .await
        .ok_or(Error::BadData)?
        .map_err(Error::Warp)?;

    // Creates binary folders if non-existent
    // If user lacks permission to create folder the binary save will fail too (this would be critical)
    let _ = tokio::fs::create_dir("bins").await;

    let binary = file
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

    let now = db::now(&mut txn).await?;
    let mut version = version
        .data()
        .await
        .ok_or(Error::BadData)?
        .map_err(Error::Warp)?;
    let buf_version = version.copy_to_bytes(version.remaining());
    let version = std::str::from_utf8(buf_version.as_ref())
        .map_err(Error::Utf8)?
        .to_uppercase();
    let filename = format!(
        "bins/{}-{:?}-{:?}-{}-{}.bin",
        version, auth.user_id, device_id, now, file_hash
    );
    let mut file = tokio::fs::File::create(&filename)
        .await
        .map_err(Error::from)?;
    // TODO: we should consider streaming the file, but we need the md5 hash do we trust the client for that?
    // TODO: fs + db is not atomic, shutdowns or panics can mess things up, we need a task to clean leaked binaries
    // TODO: using the filesystem has problems, consider a CDN, also the files are small
    // TODO: maybe insert directly in the DB to allow for atomicity and to have everything attached in one place, no loose ends
    file.write_all(&binary).await.map_err(Error::from)?;

    // TODO: storing a path in the db like this to read without care easily allows a DB hack to allow hijacking the server box too
    if let Err(err) = db::update::new(
        &mut txn,
        auth.user_id,
        device_id,
        file_hash,
        filename.clone(),
        version.to_owned(),
    )
    .await
    {
        // Best effort
        tokio::fs::remove_file(filename)
            .await
            .map_err(Error::from)?;
        return Err(err)?;
    }
    txn.commit().await.map_err(Error::from)?;
    Ok(StatusCode::OK)
}

pub async fn get(
    pool: &'static Pool,
    auth: Auth,
    headers: warp::http::HeaderMap,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    //let chip_ip = headers.get("x-ESP8266-Chip-ID");
    //let mac_address = headers.get("x-ESP8266-STA-MAC").ok_or(Error::NothingFound)?.to_str().map_err(|_|Error::NothingFound)?;
    //let ap_mac = headers.get("x-ESP8266-AP-MAC");
    //let free_space = headers.get("x-ESP8266-free-space");
    //let sketch_size = headers.get("x-ESP8266-sketch-size");
    let md5 = headers
        .get("x-ESP8266-sketch-md5")
        .ok_or(Error::NothingFound)?
        .to_str()
        .map_err(|_| Error::NothingFound)?
        .to_uppercase();
    //let chip_size = headers.get("x-ESP8266-chip-size");
    //let sdk_version = headers.get("x-ESP8266-sdk-version");

    if let Some(device_id) = auth.device_id {
        let update = match db::update::get(&mut txn, auth.user_id, device_id).await? {
            Some(update) => update,
            None => return Err(Error::NotModified)?,
        };
        if update.file_hash == md5 {
            return Err(Error::NotModified)?;
        }

        let content = tokio::fs::read(update.file_name)
            .await
            .map_err(Error::from)?;
        let md5 = md5::compute(&content);
        let md5 = &*md5;
        let mut file_hash = String::with_capacity(md5.len() * 2);
        for byte in md5 {
            write!(file_hash, "{:02X}", byte).map_err(Error::from)?;
        }
        if file_hash != update.file_hash {
            error!(
                "Binary md5 didn't match the expected: {} != {}",
                file_hash, update.file_hash
            );
            return Err(Error::CorruptBinary)?;
        }
        txn.commit().await.map_err(Error::from)?;
        Ok(http::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", content.len().to_string())
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}.bin\"", file_hash),
            )
            .header("x-MD5", file_hash)
            .body(hyper::Body::from(content)))
    } else {
        Err(Error::Forbidden)?
    }
}
