use crate::db::event::EventView;
use crate::db::firmware::Firmware;
use crate::db::sensor::MeasurementType;
use crate::extractor::Authorization;
use crate::extractor::{
    BiggestDramBlock, BiggestIramBlock, FreeDram, FreeIram, FreeStack, MacAddress, TimeRunning,
    Vcc, Version,
};
use crate::{prelude::*, CollectionId, Device, DeviceId, OrganizationId};
use crate::{Event, NewEvent};
use axum::extract::{Extension, Json, Path, TypedHeader};
use axum::http::header::{HeaderMap, HeaderName, HeaderValue};
use controllers::Result;
use serde::Serialize;
use std::iter::FromIterator;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeviceStat {
    pub version: String,
    pub time_running: u64,
    pub vcc: u16,
    pub free_dram: u64,
    pub free_iram: Option<u64>,
    pub free_stack: u32,
    pub biggest_dram_block: u64,
    pub biggest_iram_block: Option<u64>,
}

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(auth): Authorization,
    Json(event): Json<NewEvent>,
    TypedHeader(MacAddress(mac)): TypedHeader<MacAddress>,
    TypedHeader(Version(version)): TypedHeader<Version>,
    TypedHeader(TimeRunning(time_running)): TypedHeader<TimeRunning>,
    TypedHeader(Vcc(vcc)): TypedHeader<Vcc>,
    TypedHeader(FreeStack(free_stack)): TypedHeader<FreeStack>,
    TypedHeader(FreeDram(free_dram)): TypedHeader<FreeDram>,
    TypedHeader(BiggestDramBlock(biggest_dram_block)): TypedHeader<BiggestDramBlock>,
    free_iram: Option<TypedHeader<FreeIram>>,
    biggest_iram_block: Option<TypedHeader<BiggestIramBlock>>,
) -> Result<impl IntoResponse> {
    let stat = DeviceStat {
        version,
        time_running: time_running.parse()?,
        vcc: vcc.parse()?,
        free_dram: free_dram.parse()?,
        free_iram: free_iram.and_then(|TypedHeader(FreeIram(iram))| iram.parse().ok()),
        free_stack: free_stack.parse()?,
        biggest_dram_block: biggest_dram_block.parse()?,
        biggest_iram_block: biggest_iram_block
            .and_then(|TypedHeader(BiggestIramBlock(size))| size.parse().ok()),
    };

    info!(target: "event", "User: {:?}, MAC: {}, DeviceId: {:?}, Stat: {:?}", auth.user_id, mac, auth.device_id, stat);
    debug!("New Event: {:?}", event);
    if let Some(device_id) = auth.device_id {
        let mut txn = pool.begin().await?;

        let mut device = Device::find_by_id(&mut txn, &device_id).await?;
        if let Some(firmware) = Firmware::try_find_by_hash(&mut txn, &stat.version).await? {
            device.set_firmware_id(&mut txn, firmware.id()).await?;
        }

        // If there is no compiler accept whatever. This makes processing in the frontend worse as we lack metadata about types
        if let Some(compiler) = device.compiler(&mut txn).await? {
            let sensors = compiler.sensors(&mut txn).await?;
            let mut measurements = Vec::new();
            for sensor in sensors {
                let prototype = sensor.prototype(&mut txn).await?;
                measurements.extend(prototype.measurements(&mut txn).await?);
            }
            debug!("Expected Measurements: {:?}", measurements);

            let obj = event.as_object().ok_or(Error::BadData)?;
            if obj.len() != measurements.len() {
                error!("Invalid number of json arguments");
                return Err(Error::BadData);
            }
            for measurement in measurements {
                if let Some(value) = obj.get(&measurement.name) {
                    match measurement.ty {
                        MeasurementType::FloatCelsius => {
                            if let Some(value) = value.as_f64() {
                                if value < -100. || value > 100. {
                                    error!("Invalid celsius measured (-100 to 100): {}", value);
                                    return Err(Error::BadData);
                                }
                            } else {
                                error!("Invalid celsius measured: {:?}", value);
                                return Err(Error::BadData);
                            }
                        }
                        MeasurementType::RawAnalogRead => {
                            if let Some(value) = value.as_i64() {
                                if value < 0 || value > 1024 {
                                    error!("Invalid raw analog read (0-1024): {}", value);
                                    return Err(Error::BadData);
                                }
                            } else {
                                error!("Invalid raw analog read: {:?}", value);
                                return Err(Error::BadData);
                            }
                        }
                        MeasurementType::Percentage => {
                            if let Some(value) = value.as_f64() {
                                if value < 0. || value > 100. {
                                    error!("Invalid percentage: {}", value);
                                    return Err(Error::BadData);
                                }
                            } else {
                                error!("Invalid percentage measured: {:?}", value);
                                return Err(Error::BadData);
                            }
                        }
                    }
                } else {
                    error!("Missing measurement: {}", measurement.name);
                    return Err(Error::BadData);
                }
            }
        }

        Event::new(&mut txn, &device_id, event, stat.version.clone()).await?;

        // TODO: check ownership
        if let Some(firmware) = Device::update(&mut txn, device_id).await? {
            if firmware.hash() != &stat.version {
                txn.commit().await?;
                return Ok(HeaderMap::from_iter([(
                    HeaderName::from_static("latest_version"),
                    HeaderValue::from_str(firmware.hash())?,
                )]));
            }
        }
        txn.commit().await?;
        Ok(HeaderMap::new())
    } else {
        warn!(target: "event", "Not Found => User: {:?}, Device: {}", auth.user_id, mac);
        Err(Error::Forbidden)?
    }
}

type ListPath = (OrganizationId, CollectionId, DeviceId, u32);
pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
    Path((_organization_id, _collection_id, device_id, limit)): Path<ListPath>,
) -> Result<Json<Vec<EventView>>> {
    let mut txn = pool.begin().await?;
    let mut events = Vec::new();
    // TODO: check for ownership
    //let device = Device::find_by_id(&mut txn, &device_id).await?;
    //let compiler = device.compiler(&mut txn).await?;
    //if let Some(compiler) = compiler {
    //    for event in Event::list_for_compiler(&mut txn, &device_id, &compiler.id(), limit).await? {
    //        events.push(EventView::new(&mut txn, event).await?);
    //    }
    //} else {
    for event in Event::list(&mut txn, &device_id, limit).await? {
        events.push(EventView::new(&mut txn, event).await?);
    }
    //};
    txn.commit().await?;
    Ok(Json(events))
}
