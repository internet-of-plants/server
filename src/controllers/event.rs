use crate::extractor::{
    BiggestDramBlock, BiggestIramBlock, Device, FreeDram, FreeIram, FreeStack, MacAddress,
    TimeRunning, User, Vcc, Version,
};
use crate::{
    logger::*, DateTime, DeviceId, DeviceStat, Error, Event, EventView, Firmware, Pool, Result,
    SensorMeasurementType,
};
use axum::extract::{Extension, Json, Query, TypedHeader};
use axum::http::header::{HeaderMap, HeaderName, HeaderValue};
use axum::response::IntoResponse;
use handlebars::Handlebars;
use serde::Deserialize;
use serde_json::json;
use std::iter::FromIterator;

#[allow(clippy::too_many_arguments)]
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

    let collection = device.collection(&mut txn).await?;
    let organization = collection.organization(&mut txn).await?;
    if let Some(firmware) =
        Firmware::try_find_by_hash(&mut txn, &organization, &stat.version).await?
    {
        device.set_firmware(&mut txn, &firmware).await?;
    }

    if let Some(firmware) = device.update(&mut txn).await? {
        if firmware.hash() != stat.version {
            return Ok(HeaderMap::from_iter([(
                HeaderName::from_static("latest_version"),
                HeaderValue::from_str(firmware.hash())?,
            )]));
        }
    }

    // If there is no compiler accept whatever. This makes processing in the frontend worse as we lack metadata about types
    let obj = event.as_object().ok_or(Error::EventMustBeObject)?;
    if let Some(compiler) = device.compiler(&mut txn).await? {
        let sensors = compiler.sensors(&mut txn).await?;
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
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        debug!("Expected Measurements: {:?}", measurements);

        if obj.len() != measurements.len() {
            error!("Invalid number of json arguments");
            return Err(Error::MeasurementMissing);
        }
        for (ty, name) in measurements {
            if let Some(value) = obj.get(&name) {
                match ty {
                    SensorMeasurementType::FloatCelsius => {
                        if let Some(value) = value.as_f64() {
                            if !(-100. ..=100.).contains(&value) {
                                error!("Invalid celsius measured (-100 to 100): {}", value);
                                return Err(Error::MeasurementOutOfRange(
                                    value.to_string(),
                                    "-100..=100".to_owned(),
                                ));
                            }
                        } else {
                            error!("Invalid celsius measured: {:?}", value);
                            return Err(Error::InvalidMeasurementType(
                                value.clone(),
                                "f64".to_owned(),
                            ));
                        }
                    }
                    SensorMeasurementType::RawAnalogRead => {
                        if let Some(value) = value.as_i64() {
                            if !(0..=1024).contains(&value) {
                                error!("Invalid raw analog read (0-1024): {}", value);
                                return Err(Error::MeasurementOutOfRange(
                                    value.to_string(),
                                    "0..=1024".to_owned(),
                                ));
                            }
                        } else {
                            error!("Invalid raw analog read: {:?}", value);
                            return Err(Error::InvalidMeasurementType(
                                value.clone(),
                                "i64".to_owned(),
                            ));
                        }
                    }
                    SensorMeasurementType::Percentage => {
                        if let Some(value) = value.as_f64() {
                            if !(0. ..=100.).contains(&value) {
                                error!("Invalid percentage: {}", value);
                                return Err(Error::MeasurementOutOfRange(
                                    value.to_string(),
                                    "0..=100".to_owned(),
                                ));
                            }
                        } else {
                            error!("Invalid percentage measured: {:?}", value);
                            return Err(Error::InvalidMeasurementType(
                                value.clone(),
                                "f64".to_owned(),
                            ));
                        }
                    }
                }
            } else {
                error!("Missing measurement: {}", name);
                return Err(Error::MissingMeasurement(name));
            }
        }
    }

    Event::new(&mut txn, &device, event, stat).await?;

    txn.commit().await?;
    Ok(HeaderMap::new())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    device_id: DeviceId,
    since: DateTime,
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Query(request): Query<ListRequest>,
) -> Result<Json<Vec<EventView>>> {
    let mut txn = pool.begin().await?;
    let mut events = Vec::new();
    let device = crate::Device::find_by_id(&mut txn, request.device_id, &user).await?;
    for event in Event::list(&mut txn, &device, request.since).await? {
        events.push(EventView::new(event)?);
    }
    txn.commit().await?;
    Ok(Json(events))
}
