use crate::db::sensor::measurement::{Measurement, MeasurementType, MeasurementKind};
use crate::db::sensor::{
    config_request::NewConfigRequest, config_type::WidgetKind,
};
use crate::db::target::Target;
use crate::db::target_prototype::TargetPrototype;
use crate::prelude::*;

use super::SensorPrototype;

pub async fn create_builtin(txn: &mut Transaction<'_>) -> Result<()> {
    let esp8266_target_prototype = esp8266_target(&mut *txn).await?;
    nodemcuv2_esp8266_target(&mut *txn, &esp8266_target_prototype).await?;

    let esp32_target_prototype = esp32_target(&mut *txn).await?;
    esp32dev_esp32_target(&mut *txn, &esp32_target_prototype).await?;

    let posix_target_prototype = posix_target(&mut *txn).await?;
    native_posix_target(&mut *txn, &posix_target_prototype).await?;

    dht(&mut *txn).await?;
    soil_resistivity(&mut *txn).await?;
    factory_reset_button(&mut *txn).await?;
    dallas_temperature(&mut *txn).await?;

    Ok(())
}

async fn dht(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    Ok(SensorPrototype::new(
        txn,
        "DHT".to_owned(),
        vec!["https://github.com/internet-of-plants/dht".to_owned()],
        vec!["dht.hpp".to_owned()],
        vec![
            "static dht::Dht airTempAndHumidity{{index}}(IOP_PIN_RAW(config::airTempAndHumidity{{index}}), config::dhtVersion{{index}});".to_owned(),
        ],
        vec![
            "airTempAndHumidity{{index}}.begin();".to_owned()
        ],
        vec![
            Measurement {
                human_name: "Air Temperature".to_owned(),
                name: "air_temperature_celsius{{index}}".to_owned(),
                value: "airTempAndHumidity{{index}}.measureTemperature();".to_owned(),
                ty: MeasurementType::FloatCelsius,
                kind: MeasurementKind::AirTemperature,
            },
            Measurement {
                human_name: "Air Humidity".to_owned(),
                name: "air_humidity_percentage{{index}}".to_owned(),
                value: "airTempAndHumidity{{index}}.measureHumidity();".to_owned(),
                ty: MeasurementType::Percentage,
                kind: MeasurementKind::AirHumidity,
            },
        ],
        vec![
            NewConfigRequest::new("Data Input".to_owned(), "airTempAndHumidity{{index}}".to_owned(), "Pin".to_owned(), WidgetKind::PinSelection),
            NewConfigRequest::new("Model".to_owned(), "dhtVersion{{index}}".to_owned(), "dht::Version".to_owned(), WidgetKind::Selection(vec![
                "dht::Version::DHT11".to_owned(),
                "dht::Version::DHT12".to_owned(),
                "dht::Version::DHT21".to_owned(),
                "dht::Version::DHT22".to_owned(),
                "dht::Version::AM2301".to_owned(),
            ])),
        ],
    ).await?)
}

async fn dallas_temperature(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    Ok(SensorPrototype::new(
        txn,
        "Dallas Temperature".to_owned(),
        vec!["https://github.com/internet-of-plants/dallas_temperature".to_owned()],
        vec!["dallas_temperature.hpp".to_owned()],
        vec![
            "static dallas::TemperatureCollection soilTemperature{{index}}(IOP_PIN_RAW(config::soilTemperature{{index}}));".to_owned(),
        ],
        vec![
            "soilTemperature{{index}}.begin();".to_owned(),
        ],
        vec![
            Measurement {
                human_name: "Soil Temperature".to_owned(),
                name: "soil_temperature_celsius{{index}}".to_owned(),
                value: "soilTemperature{{index}}.measure();".to_owned(),
                ty: MeasurementType::FloatCelsius,
                kind: MeasurementKind::SoilTemperature,
            },
        ],
        vec![
            NewConfigRequest::new("Data Input".to_owned(), "soilTemperature{{index}}".to_owned(), "Pin".to_owned(), WidgetKind::PinSelection),
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
            "reset::setup(IOP_PIN_RAW(config::factoryResetButton{{index}}));".to_owned(),
            "loop.setInterval(1000, reset::resetIfNeeded);".to_owned(),
        ],
        vec![],
        vec![NewConfigRequest::new(
            "Button".to_owned(),
            "factoryResetButton{{index}}".to_owned(),
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
            "static sensor::SoilResistivity soilResistivity{{index}}(IOP_PIN_RAW(config::soilResistivityPower{{index}}));".to_owned(),
        ],
        vec![
            "soilResistivity{{index}}.begin();".to_owned(),
        ],
        vec![
            Measurement {
                human_name: "Soil Resistivity Raw".to_owned(),
                name: "soil_resistivity_raw{{index}}".to_owned(),
                value: "soilResistivity{{index}}.measure();".to_owned(),
                ty: MeasurementType::RawAnalogRead,
                kind: MeasurementKind::SoilMoisture,
            },
        ],
        vec![
            // TODO: we should configure the analog pin here too
            NewConfigRequest::new("Power".to_owned(), "soilResistivityPower{{index}}".to_owned(), "Pin".to_owned(), WidgetKind::PinSelection),
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
        Some("framework-arduinoespressif8266 @ https://github.com/esp8266/Arduino.git#d5444c4aa38bff01269cfbd98a13a1454d0c62df".to_owned()),
        Some("monitor_filters = esp8266_exception_decoder\nboard_build.f_cpu = 160000000L\nmonitor_speed = 115200".to_owned()),
        Some("deep".to_owned())
    ).await?)
}

async fn nodemcuv2_esp8266_target(
    txn: &mut Transaction<'_>,
    target_prototype: &TargetPrototype,
) -> Result<Target> {
    Ok(Target::new(
        txn,
        Some("nodemcuv2".to_owned()),
        vec![
            "Pin::D1".to_owned(),
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
        target_prototype,
    )
    .await?)
}

async fn esp32_target(txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
    let mut prototype = TargetPrototype::new(
        txn,
        "esp32".to_owned(),
        "-D IOP_ESP32\n    -D IOP_SSL".to_owned(),
        "https://github.com/platformio/platform-espressif32.git#378419806ce465505a36437157d32c17144e45d2".to_owned(),
        Some("arduino".to_owned()),
        Some("toolchain-xtensa-esp32 @ 8.4.0+2021r1
    espressif/toolchain-riscv32-esp @ 8.4.0+2021r1
    framework-arduinoespressif32 @ https://github.com/internet-of-plants/arduino-esp32.git#release_2011
    platformio/tool-esptoolpy @ https://github.com/tasmota/esptool/releases/download/v3.2/esptool-v3.2.zip".to_owned()),
    Some("board_build.mcu = esp32\nboard_build.f_cpu = 240000000L\nmonitor_speed = 115200".to_owned()),
        Some("deep".to_owned())
    ).await?;
    prototype
        .set_build_unflags(txn, Some("-std=gnu++11".to_owned()))
        .await?;
    Ok(prototype)
}

async fn esp32dev_esp32_target(
    txn: &mut Transaction<'_>,
    target_prototype: &TargetPrototype,
) -> Result<Target> {
    Ok(Target::new(
        txn,
        Some("esp32dev".to_owned()),
        vec![],
        "".to_owned(),
        target_prototype,
    )
    .await?)
}

async fn posix_target(txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
    let prototype = TargetPrototype::new(
        txn,
        "posix".to_owned(),
        "-Wextra
    -Wpedantic
    -pedantic-errors
    -fstack-protector
    -Wstack-protector"
            .to_owned(),
        "native".to_owned(),
        None,
        None,
        None,
        None,
    )
    .await?;
    Ok(prototype)
}

async fn native_posix_target(
    txn: &mut Transaction<'_>,
    target_prototype: &TargetPrototype,
) -> Result<Target> {
    let mut target = Target::new(
        txn,
        None,
        vec![
            "Pin::D1".to_owned(),
            "Pin::D2".to_owned(),
            "Pin::D5".to_owned(),
            "Pin::D6".to_owned(),
            "Pin::D7".to_owned(),
        ],
        "#ifndef PIN_HPP
#define PIN_HPP

/// Dummy pin mapping for POSIX mock
enum class Pin { D1 = 5, D2 = 4, D5 = 14, D6 = 12, D7 = 13 };

#endif"
            .to_owned(),
        target_prototype,
    )
    .await?;
    target
        .set_build_flags(
            txn,
            Some(
                "
    -D IOP_POSIX_MOCK
    -D IOP_USERNAME=\"admin\"
    -D IOP_PASSWORD=\"admin\""
                    .to_owned(),
            ),
        )
        .await?;
    Ok(target)
}
