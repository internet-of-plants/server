use crate::db::event::EventView;
use crate::db::firmware::Firmware;
use crate::db::sensor::measurement::MeasurementType;
use crate::extractor::{
    BiggestDramBlock, BiggestIramBlock, Device, FreeDram, FreeIram, FreeStack, MacAddress,
    TimeRunning, User, Vcc, Version,
};
use crate::prelude::*;
use crate::{DeviceId, Event};
use axum::extract::{Extension, Json, Query, TypedHeader};
use axum::http::header::{HeaderMap, HeaderName, HeaderValue};
use controllers::Result;
use handlebars::Handlebars;
use serde::{Deserialize, Serialize};
use serde_json::json;
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
    Device(mut device): Device,
    Json(event): Json<serde_json::Value>,
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

    info!(target: "event", "MAC: {}, DeviceId: {:?}, Stat: {:?}", mac, device, stat);
    debug!("New Event: {:?}", event);
    let mut txn = pool.begin().await?;

    if let Some(firmware) = Firmware::try_find_by_hash(&mut txn, &stat.version).await? {
        device.set_firmware_id(&mut txn, firmware.id()).await?;
    }

    // TODO: check ownership
    if let Some(firmware) = device.update(&mut txn).await? {
        if firmware.hash() != &stat.version {
            return Ok(HeaderMap::from_iter([(
                HeaderName::from_static("latest_version"),
                HeaderValue::from_str(firmware.hash())?,
            )]));
        }
    }

    // If there is no compiler accept whatever. This makes processing in the frontend worse as we lack metadata about types
    let obj = event.as_object().ok_or(Error::BadData)?;
    if let Some(compiler) = device.compiler(&mut txn).await? {
        let sensors = compiler.sensors(&mut txn, &device).await?;
        let mut measurements = Vec::new();
        for (index, sensor) in sensors.into_iter().enumerate() {
            let prototype = sensor.prototype;
            measurements.extend(
                prototype
                    .measurements
                    .into_iter()
                    .map(|m| {
                        let reg = Handlebars::new();
                        let name = reg.render_template(&m.name, &json!({ "index": index }))?;
                        Ok((m.ty, name))
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
            );
        }
        debug!("Expected Measurements: {:?}", measurements);

        if obj.len() != measurements.len() {
            error!("Invalid number of json arguments");
            return Err(Error::BadData);
        }
        for (ty, name) in measurements {
            if let Some(value) = obj.get(&name) {
                match ty {
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
                error!("Missing measurement: {}", name);
                return Err(Error::BadData);
            }
        }
    }

    Event::new(&mut txn, &device, event, stat.version.clone()).await?;

    txn.commit().await?;
    Ok(HeaderMap::new())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    device_id: DeviceId,
    limit: u32,
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Query(request): Query<ListRequest>,
) -> Result<Json<Vec<EventView>>> {
    let mut txn = pool.begin().await?;
    let mut events = Vec::new();
    let device = db::device::Device::find_by_id(&mut txn, request.device_id, &user).await?;
    for event in Event::list(&mut txn, &device, request.limit).await? {
        events.push(EventView::new(event)?);
    }
    txn.commit().await?;
    Ok(Json(events))
}
