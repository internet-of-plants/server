use crate::{extractor::Device, extractor::Esp8266Md5, logger::*, Error, Pool, Result};
use axum::{body::Bytes, body::Full, response::IntoResponse, Extension, TypedHeader};
use std::fmt::Write;

pub async fn update(
    Extension(pool): Extension<&'static Pool>,
    Device(device): Device,
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

    let firmware = match device.update(&mut txn).await? {
        Some(update) => update,
        None => return Err(Error::NoBinaryAvailable)?,
    };
    if firmware.hash() == md5 {
        return Err(Error::NoUpdateAvailable)?;
    }

    let hash = firmware.hash().to_owned();
    if let Some(binary) = firmware.bin(&mut txn).await? {
        let md5 = md5::compute(&binary);
        let md5 = &*md5;
        let mut file_hash = String::with_capacity(md5.len() * 2);
        for byte in md5 {
            write!(file_hash, "{:02X}", byte)?;
        }
        if file_hash != hash {
            error!(
                "Binary md5 didn't match the expected: {} != {}",
                file_hash, hash,
            );
            return Err(Error::CorruptedBinary)?;
        }
        txn.commit().await?;
        let response = axum::http::Response::builder()
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", binary.len().to_string())
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{}.bin\"", file_hash),
            )
            .header("x-MD5", file_hash)
            .body(Full::new(Bytes::from(binary)))?;
        Ok(response)
    } else {
        Err(Error::MissingBinary)?
    }
}
