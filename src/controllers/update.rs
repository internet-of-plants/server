use crate::db::firmware::Firmware;
use crate::prelude::*;
use crate::{DeviceId, Update};
//use bytes::{Buf, BufMut};
use controllers::Result;
//use futures::{StreamExt, TryStreamExt};
use std::fmt::Write;
//use warp::filters::multipart::FormData;
use crate::extractor::{Authorization, Esp8266Md5};
use axum::extract::{Extension, Form, Path, TypedHeader};
use axum::{body::Bytes, body::Full, http::StatusCode};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UpdateForm {
    version: String,
    file: Vec<u8>,
}

pub async fn new(
    Path(device_id): Path<DeviceId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    Form(form): Form<UpdateForm>,
) -> Result<StatusCode> {
    let mut txn = pool.begin().await?;

    //db::plant::owns(&mut txn, auth.user_id, device_id).await?;

    //let mut version = form
    //    .next()
    //    .await
    //    .ok_or(Error::BadData)?
    //    .map_err(Error::Warp)?;
    //let mut version = version
    //    .data()
    //    .await
    //    .ok_or(Error::BadData)?
    //    .map_err(Error::Warp)?;
    //let buf_version = version.copy_to_bytes(version.remaining());
    //let version = std::str::from_utf8(buf_version.as_ref())
    //    .map_err(Error::Utf8)?
    //    .to_uppercase();

    //let file = form
    //    .next()
    //    .await
    //    .ok_or(Error::BadData)?
    //    .map_err(Error::Warp)?;
    //let binary = file
    //    .stream()
    //    .try_fold(Vec::new(), |mut vec, data| {
    //        vec.put(data);
    //        async move { Ok(vec) }
    //    })
    //    .await
    //    .map_err(Error::from)?;

    let firmware = Firmware::new(&mut txn, None, form.file).await?;
    let _update = Update::new(
        &mut txn,
        auth.user_id,
        device_id,
        firmware.id(),
        form.version,
    )
    .await?;

    txn.commit().await?;
    Ok(StatusCode::OK)
}

pub async fn get(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    TypedHeader(Esp8266Md5(md5)): TypedHeader<Esp8266Md5>,
) -> Result<impl IntoResponse> {
    let mut txn = pool.begin().await?;

    //let chip_ip = headers.get("x-ESP8266-Chip-ID");
    //let mac_address = headers.get("x-ESP8266-STA-MAC").ok_or(Error::NothingFound)?.to_str().map_err(|_|Error::NothingFound)?;
    //let ap_mac = headers.get("x-ESP8266-AP-MAC");
    //let free_space = headers.get("x-ESP8266-free-space");
    //let sketch_size = headers.get("x-ESP8266-sketch-size");
    let md5 = md5.to_uppercase();
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
        txn.commit().await?;
        let response = axum::http::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", firmware.binary().len().to_string())
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}.bin\"", file_hash),
            )
            .header("x-MD5", file_hash)
            .body(Full::new(Bytes::from(firmware.into_binary())))?;
        Ok(response)
    } else {
        Err(Error::Forbidden)?
    }
}
