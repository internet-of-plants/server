use crate::db::firmware::Firmware;
use crate::extractor::{Authorization, Esp8266Md5};
use crate::{prelude::*, Device};
use crate::{CollectionId, DeviceId, OrganizationId, Update};
use axum::extract::{Extension, Multipart, Path, TypedHeader};
use axum::{body::Bytes, body::Full, http::StatusCode};
use controllers::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUpdate {
    pub version: String,
    pub file: Vec<u8>,
}

type NewPath = (OrganizationId, CollectionId, DeviceId);
pub async fn new(
    Path((_organization_id, _collection_id, device_id)): Path<NewPath>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    mut multipart: Multipart,
) -> Result<StatusCode> {
    if auth.device_id.is_some() && Some(device_id) != auth.device_id {
        return Err(Error::Forbidden);
    }

    let mut txn = pool.begin().await?;

    //db::plant::owns(&mut txn, auth.user_id, device_id).await?;

    let mut map: HashMap<String, Bytes> = HashMap::default();
    println!("{:?}", multipart);
    while let Some(field) = multipart.next_field().await? {
        println!("aaaa {:?}", field);
        println!("name");
        let name = field.name().ok_or(Error::BadData)?.to_string();
        println!("data");
        let data = field.bytes().await?;
        map.insert(name, data);
    }

    dbg!(map.keys().collect::<Vec<_>>());
    let firmware = Firmware::new(
        &mut txn,
        None,
        map.get("binary").ok_or(Error::BadData)?.to_vec(),
    )
    .await?;
    let _update = Update::new(
        &mut txn,
        auth.user_id,
        device_id,
        firmware.id(),
        String::from_utf8_lossy(map.get("version").ok_or(Error::BadData)?).to_string(),
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
        let firmware = match Device::update(&mut txn, device_id).await? {
            Some(update) => update,
            None => return Err(Error::NotModified)?,
        };
        if firmware.hash() == md5 {
            return Err(Error::NotModified)?;
        }

        let hash = firmware.hash().to_owned();
        if let Some(binary) = firmware.into_binary() {
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
                return Err(Error::CorruptBinary)?;
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
            // Updates should always have a binary attached to it
            Err(Error::InvalidUpdateFound)
        }
    } else {
        Err(Error::Forbidden)?
    }
}
