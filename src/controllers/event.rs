use crate::prelude::*;
use crate::{Event, NewEvent, Update};
use controllers::Result;

fn optional_parse_header<T: std::str::FromStr>(
    headers: &warp::http::HeaderMap,
    key: &'static str,
) -> Result<Option<T>, Error> {
    Ok(headers
        .get(key)
        .map(|k| {
            k.to_str()
                .map_err(|_| Error::MissingHeader(key))
                .and_then(|k| {
                    k.to_string()
                        .parse::<T>()
                        .map_err(|_| Error::MissingHeader(key))
                })
        })
        .transpose()?)
}

fn parse_header<T: std::str::FromStr>(
    headers: &warp::http::HeaderMap,
    key: &'static str,
) -> Result<T, Error> {
    optional_parse_header(headers, key)?.ok_or(Error::MissingHeader(key))
}

pub async fn new(
    pool: &'static Pool,
    auth: Auth,
    event: NewEvent,
    headers: warp::http::HeaderMap,
) -> Result<impl Reply> {
    let mac: String = parse_header(&headers, "MAC_ADDRESS")?;
    let stat = DeviceStat {
        version: parse_header(&headers, "VERSION")?,
        time_running: parse_header(&headers, "TIME_RUNNING")?,
        vcc: parse_header(&headers, "VCC")?,
        free_dram: parse_header(&headers, "FREE_DRAM")?,
        free_iram: optional_parse_header(&headers, "FREE_IRAM")?,
        free_stack: parse_header(&headers, "FREE_STACK")?,
        biggest_dram_block: parse_header(&headers, "BIGGEST_BLOCK_DRAM")?,
        biggest_iram_block: optional_parse_header(&headers, "BIGGEST_BLOCK_IRAM")?,
    };

    if let Some(device_id) = auth.device_id {
        let mut txn = pool.begin().await.map_err(Error::from)?;

        info!(target: "event", "User: {:?}, MAC: {}, DeviceId: {:?}, Stat: {:?}", auth.user_id, mac, device_id, stat);
        Event::new(&mut txn, &device_id, event, stat.version.clone()).await?;

        if let Some(update) = Update::find_by_device(&mut txn, auth.user_id, device_id).await? {
            if update.file_hash() != &stat.version {
                txn.commit().await.map_err(Error::from)?;
                return Ok(http::Response::builder()
                    .header("LATEST_VERSION", update.file_hash())
                    .body(""));
            }
        }
        txn.commit().await.map_err(Error::from)?;
        Ok(http::Response::builder().body(""))
    } else {
        warn!(target: "event", "Not Found => User: {:?}, Device: {}", auth.user_id, mac);
        Err(Error::Forbidden)?
    }
}
