use crate::db::firmware::Firmware;
use crate::prelude::*;
use crate::{DeviceId, Update};
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
    let mut version = version
        .data()
        .await
        .ok_or(Error::BadData)?
        .map_err(Error::Warp)?;
    let buf_version = version.copy_to_bytes(version.remaining());
    let version = std::str::from_utf8(buf_version.as_ref())
        .map_err(Error::Utf8)?
        .to_uppercase();

    let file = form
        .next()
        .await
        .ok_or(Error::BadData)?
        .map_err(Error::Warp)?;
    let binary = file
        .stream()
        .try_fold(Vec::new(), |mut vec, data| {
            vec.put(data);
            async move { Ok(vec) }
        })
        .await
        .map_err(Error::from)?;

    let firmware = Firmware::new(&mut txn, None, binary).await?;
    let _update = Update::new(
        &mut txn,
        auth.user_id,
        device_id,
        firmware.id(),
        version.to_owned(),
    )
    .await?;

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
        let update = match Update::find_by_device(&mut txn, auth.user_id, device_id).await? {
            Some(update) => update,
            None => return Err(Error::NotModified)?,
        };
        let firmware = update.firmware(&mut txn).await?;
        if firmware.hash() == md5 {
            return Err(Error::NotModified)?;
        }

        let md5 = md5::compute(firmware.binary());
        let md5 = &*md5;
        let mut file_hash = String::with_capacity(md5.len() * 2);
        for byte in md5 {
            write!(file_hash, "{:02X}", byte).map_err(Error::from)?;
        }
        if file_hash != firmware.hash() {
            error!(
                "Binary md5 didn't match the expected: {} != {}",
                file_hash,
                firmware.hash(),
            );
            return Err(Error::CorruptBinary)?;
        }
        txn.commit().await.map_err(Error::from)?;
        Ok(http::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", firmware.binary().len().to_string())
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}.bin\"", file_hash),
            )
            .header("x-MD5", file_hash)
            .body(hyper::Body::from(firmware.into_binary())))
    } else {
        Err(Error::Forbidden)?
    }
}
