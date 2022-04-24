use crate::db::board::Board;
use crate::db::sensor::{config_request::NewConfigRequest, config_type::WidgetKind, Measurement};
use crate::db::target_prototype::{TargetPrototype, TargetPrototypeId};
use crate::prelude::*;

use super::SensorPrototype;

pub async fn create_builtin(txn: &mut Transaction<'_>) -> Result<()> {
    let esp8266_target = esp8266_target(&mut *txn).await?;
    nodemcuv2_esp8266_board(&mut *txn, esp8266_target.id).await?;

    dht(&mut *txn).await?;
    soil_resistivity(&mut *txn).await?;
    factory_reset_button(&mut *txn).await?;
    soil_temperature(&mut *txn).await?;

    Ok(())
}

async fn dht(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    Ok(SensorPrototype::new(
        txn,
        "DHT".to_owned(),
        vec!["https://github.com/internet-of-plants/dht".to_owned()],
        vec!["dht.hpp".to_owned()],
        vec![
            "static dht::Dht airTempAndHumidity(IOP_PIN_RAW(config::airTempAndHumidity), config::dhtVersion);".to_owned(),
        ],
        vec![
            "airTempAndHumidity.begin();".to_owned()
        ],
        vec![
            Measurement {
                name: "air_temperature_celsius".to_owned(),
                value: "airTempAndHumidity.measureTemperature();".to_owned(),
            },
            Measurement {
                name: "air_humidity_percentage".to_owned(),
                value: "airTempAndHumidity.measureHumidity();".to_owned(),
            },
            Measurement {
                name: "air_heat_index_celsius".to_owned(),
                value: "airTempAndHumidity.measureHeatIndex();".to_owned(),
            },
        ],
        vec![
            NewConfigRequest::new("airTempAndHumidity".to_owned(), "Pin".to_owned(), WidgetKind::PinSelection),
            NewConfigRequest::new("dhtVersion".to_owned(), "dht::Version".to_owned(), WidgetKind::Selection(vec![
                "dht::Version::DHT11".to_owned(),
                "dht::Version::DHT12".to_owned(),
                "dht::Version::DHT21".to_owned(),
                "dht::Version::DHT22".to_owned(),
                "dht::Version::AM2301".to_owned(),
            ])),
        ],
    ).await?)
}

async fn soil_temperature(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    Ok(SensorPrototype::new(
        txn,
        "Soil Temperature".to_owned(),
        vec!["https://github.com/internet-of-plants/soil_temperature".to_owned()],
        vec!["dallas_temperature.hpp".to_owned()],
        vec![
            "static dallas::TemperatureCollection soilTemperature(IOP_PIN_RAW(config::soilTemperature));".to_owned(),
        ],
        vec![
            "soilTemperature.begin();".to_owned(),
        ],
        vec![
            Measurement {
                name: "soil_temperature_celsius".to_owned(),
                value: "soilTemperature.measure();".to_owned(),
            },
        ],
        vec![
            NewConfigRequest::new("soilTemperature".to_owned(), "Pin".to_owned(), WidgetKind::PinSelection),
        ],
    ).await?)
}

async fn factory_reset_button(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    Ok(SensorPrototype::new(
        txn,
        "Factory Reset Button".to_owned(),
        vec!["https://github.com/internet-of-plants/factory_reset_button".to_owned()],
        vec!["factory_reset_button.hpp".to_owned()],
        vec![],
        vec![
            "reset::setup(IOP_PIN_RAW(config::factoryResetButton));".to_owned(),
            "loop.setInterval(1000, reset::resetIfNeeded);".to_owned(),
        ],
        vec![],
        vec![NewConfigRequest::new(
            "factoryResetButton".to_owned(),
            "Pin".to_owned(),
            WidgetKind::PinSelection,
        )],
    )
    .await?)
}

async fn soil_resistivity(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    Ok(SensorPrototype::new(
        txn,
        "Soil Resistivity".to_owned(),
        vec!["https://github.com/internet-of-plants/soil_resistivity".to_owned()],
        vec!["soil_resistivity.hpp".to_owned()],
        vec![
            "static sensor::SoilResistivity soilResistivity(IOP_PIN_RAW(config::soilResistivityPower));".to_owned(),
        ],
        vec![
            "soilResistivity.begin();".to_owned(),
        ],
        vec![
            Measurement {
                name: "soil_resistivity_raw".to_owned(),
                value: "soilResistivity.measure();".to_owned(),
            },
        ],
        vec![
            NewConfigRequest::new("soilResistivityPower".to_owned(), "Pin".to_owned(), WidgetKind::PinSelection),
        ],
    ).await?)
}

async fn esp8266_target(txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
    Ok(TargetPrototype::new(
        txn,
        "esp8266".to_owned(),
	    "-D IOP_ESP8266\n    -D IOP_SSL".to_owned(),
        "espressif8266".to_owned(),
        Some("arduino".to_owned()),
        "framework-arduinoespressif8266 @ https://github.com/esp8266/Arduino.git#d5444c4aa38bff01269cfbd98a13a1454d0c62df".to_owned(),
        "monitor_filters = esp8266_exception_decoder\nboard_build.f_cpu = 160000000L\nmonitor_speed = 115200".to_owned(),
        Some("deep".to_owned())
    ).await?)
}

async fn nodemcuv2_esp8266_board(
    txn: &mut Transaction<'_>,
    target_id: TargetPrototypeId,
) -> Result<Board> {
    Ok(Board::new(
        txn,
        "nodemcuv2".to_owned(),
        vec!["Pin::D1".to_owned(),
            "Pin::D2".to_owned(),
            "Pin::D5".to_owned(),
            "Pin::D6".to_owned(),
            "Pin::D7".to_owned(),
        ],
        "#ifndef PIN_HPP
#define PIN_HPP

/// Pin mapping for ESP8266
enum class Pin { D1 = 5, D2 = 4, D5 = 14, D6 = 12, D7 = 13 };

#endif"
            .to_owned(),
        target_id,
    )
    .await?)
}
