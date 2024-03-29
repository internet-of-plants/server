use crate::extractor::{
    BiggestDramBlock, BiggestIramBlock, Device, FreeDram, FreeIram, FreeStack, MacAddress,
    TimeRunning, User, Vcc, Version,
};
use crate::{
    logger::*, Collection, DateTime, DeviceId, DeviceStat, Error, Event, EventView, Firmware, Pool,
    Result, SensorMeasurementType, Transaction,
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
        version: version.to_lowercase(),
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

    let mut collection = device.collection(&mut txn).await?;

    // Don't even process request if there is an update
    if let Some(firmware) = collection.update(&mut txn).await? {
        if firmware.binary_hash() != stat.version() {
            return Ok(HeaderMap::from_iter([(
                HeaderName::from_static("latest_version"),
                HeaderValue::from_str(&firmware.binary_hash().to_lowercase())?,
            )]));
        }
    }

    let organization = collection.organization(&mut txn).await?;
    if let Some(firmware) =
        Firmware::try_find_by_hash(&mut txn, &organization, stat.version()).await?
    {
        if collection.compiler(&mut txn).await?.is_none() {
            if let Some(compilation) = firmware.compilation(&mut txn).await? {
                let compiler = compilation.compiler(&mut txn).await?;
                if let Some(col) = Collection::find_by_compiler(&mut txn, &compiler).await? {
                    if col.target_prototype_id() != device.target_prototype_id() {
                        return Err(Error::WrongTargetPrototype(
                            col.target_prototype_id(),
                            device.target_prototype_id(),
                        ));
                    }

                    device.set_collection(&mut txn, &col).await?;
                    collection = col;
                } else {
                    let target = compiler.target(&mut txn).await?;
                    let prototype = target.prototype(&mut txn).await?;
                    if collection.target_prototype_id() != prototype.id() {
                        return Err(Error::WrongTargetPrototype(
                            collection.target_prototype_id(),
                            prototype.id(),
                        ));
                    }

                    collection.set_compiler(&mut txn, Some(&compiler)).await?;
                }
            } else {
                // Assume all devices with the same firmware are of the same collection, a race might make this not true, but let's pick one
                if let Some(dev) =
                    crate::Device::list_by_firmware(&mut txn, &firmware, &organization)
                        .await?
                        .pop()
                {
                    let col = dev.collection(&mut txn).await?;

                    if col.target_prototype_id() != device.target_prototype_id() {
                        return Err(Error::WrongTargetPrototype(
                            col.target_prototype_id(),
                            device.target_prototype_id(),
                        ));
                    }
                    device.set_collection(&mut txn, &col).await?;
                }
            }
        }
        device.set_firmware(&mut txn, &firmware).await?;
    }

    if !event.is_null() {
        handle_measurements(&mut txn, &collection, &device, stat.clone(), event).await?;
    }

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

async fn handle_measurements(
    txn: &mut Transaction<'_>,
    collection: &Collection,
    device: &crate::Device,
    stat: DeviceStat,
    mut event: serde_json::Value,
) -> Result<Event> {
    let obj = event.as_object_mut().ok_or(Error::EventMustBeObject)?;

    // If there is no compiler accept whatever. This makes processing in the frontend worse as we lack metadata about types
    if let Some(compiler) = collection.compiler(txn).await? {
        let sensors = compiler.sensors(txn).await?;
        let mut measurements = Vec::new();
        for sensor in sensors {
            let index = sensor.index();
            let prototype = sensor.prototype();
            measurements.extend(
                prototype
                    .measurements()
                    .iter()
                    .cloned()
                    .map(|m| {
                        let reg = Handlebars::new();
                        let name =
                            reg.render_template(m.variable_name(), &json!({ "index": index }))?;
                        Ok((m.ty().clone(), name))
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
            if let Some(value) = obj.get_mut(&name) {
                match ty {
                    SensorMeasurementType::FloatCelsius => {
                        // There is no NaN in JSON, most serializers cast it to null
                        if value.as_null().is_some() {
                            *value = serde_json::json!(f64::NAN);
                        }

                        if value.as_f64().is_none() {
                            error!("Invalid celsius measured: {:?}", value);
                            return Err(Error::InvalidMeasurementType(
                                value.clone(),
                                "f64".to_owned(),
                            ));
                        }
                    }
                    SensorMeasurementType::RawAnalogRead => {
                        if value.as_i64().is_none() {
                            error!("Invalid raw analog read: {:?}", value);
                            return Err(Error::InvalidMeasurementType(
                                value.clone(),
                                "i64".to_owned(),
                            ));
                        }
                    }
                    SensorMeasurementType::Percentage => {
                        // There is no NaN in JSON, most serializers cast it to null
                        if value.as_null().is_some() {
                            *value = serde_json::json!(f64::NAN);
                        }

                        if value.as_f64().is_none() {
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

    Event::new(txn, device, event, stat).await
}
