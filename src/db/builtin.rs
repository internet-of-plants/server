use crate::{
    Definition, Dependency, DeviceWidgetKind, NewDeviceConfigRequest, NewSensorConfigRequest,
    NewSensorWidgetKind, Result, SecretAlgo, SensorMeasurement, SensorMeasurementKind,
    SensorMeasurementType, SensorPrototype, SensorReference, Target, TargetPrototype, Transaction,
};

pub async fn create_builtin(txn: &mut Transaction<'_>) -> Result<()> {
    let esp8266_target_prototype = esp8266_target_prototype(txn).await?;
    nodemcuv2_esp8266_target(txn, &esp8266_target_prototype).await?;

    let esp32_target_prototype = esp32_target_prototype(txn).await?;
    esp32dev_esp32_target(txn, &esp32_target_prototype).await?;

    let linux_target_prototype = linux_target_prototype(txn).await?;
    native_linux_target(txn, &linux_target_prototype).await?;
    native_linux_local_debugging_target(txn, &linux_target_prototype).await?;

    dht(txn).await?;
    soil_resistivity(txn).await?;
    factory_reset_button(txn).await?;
    light_relay(txn).await?;
    cooler(txn).await?;
    dallas_temperature(txn).await?;
    water_pump(txn).await?;

    Ok(())
}

async fn dht(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    SensorPrototype::new(
        txn,
        "DHT".to_owned(),
        "airTempAndHumidity",
        vec![Dependency::new("https://github.com/internet-of-plants/dht", "main")],
        vec!["dht.hpp".to_owned()],
        vec![
            "static dht::Dht airTempAndHumidity{{index}}(IOP_PIN_RAW(config::airTempAndHumidity{{index}}), config::dhtVersion{{index}});".to_owned(),
        ],
        vec![
            "airTempAndHumidity{{index}}.begin();".to_owned()
        ],
        vec![
        ],
        vec![
            SensorMeasurement {
                human_name: "Air Temperature".to_owned(),
                name: "air_temperature_celsius{{index}}".to_owned(),
                value: "airTempAndHumidity{{index}}.measureTemperature();".to_owned(),
                ty: SensorMeasurementType::FloatCelsius,
                kind: SensorMeasurementKind::AirTemperature,
            },
            SensorMeasurement {
                human_name: "Air Humidity".to_owned(),
                name: "air_humidity_percentage{{index}}".to_owned(),
                value: "airTempAndHumidity{{index}}.measureHumidity();".to_owned(),
                ty: SensorMeasurementType::Percentage,
                kind: SensorMeasurementKind::AirHumidity,
            },
        ],
        vec![
            NewSensorConfigRequest::new("Port".to_owned(), "airTempAndHumidity{{index}}".to_owned(), "Pin".to_owned(), NewSensorWidgetKind::PinSelection),
            NewSensorConfigRequest::new("Model".to_owned(), "dhtVersion{{index}}".to_owned(), "dht::Version".to_owned(), NewSensorWidgetKind::Selection(vec![
                "dht::Version::DHT11".to_owned(),
                "dht::Version::DHT12".to_owned(),
                "dht::Version::DHT21".to_owned(),
                "dht::Version::DHT22".to_owned(),
                "dht::Version::AM2301".to_owned(),
            ])),
        ],
    ).await
}

async fn dallas_temperature(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    SensorPrototype::new(
        txn,
        "Dallas Temperature".to_owned(),
        "soilTemperature",
        vec![Dependency::new("https://github.com/internet-of-plants/dallas-temperature", "main")],
        vec!["dallas_temperature.hpp".to_owned()],
        vec![
            "static dallas::TemperatureCollection soilTemperature{{index}}(IOP_PIN_RAW(config::soilTemperature{{index}}));".to_owned(),
        ],
        vec![
            "soilTemperature{{index}}.begin();".to_owned(),
        ],
        vec![

        ],
        vec![
            SensorMeasurement {
                human_name: "Soil Temperature".to_owned(),
                name: "soil_temperature_celsius{{index}}".to_owned(),
                value: "soilTemperature{{index}}.measure();".to_owned(),
                ty: SensorMeasurementType::FloatCelsius,
                kind: SensorMeasurementKind::SoilTemperature,
            },
        ],
        vec![
            NewSensorConfigRequest::new("Port".to_owned(), "soilTemperature{{index}}".to_owned(), "Pin".to_owned(), NewSensorWidgetKind::PinSelection),
        ],
    ).await
}

async fn factory_reset_button(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    SensorPrototype::new(
        txn,
        "Factory Reset Button".to_owned(),
        None,
        vec![Dependency::new(
            "https://github.com/internet-of-plants/factory-reset-button",
            "main",
        )],
        vec!["factory_reset_button.hpp".to_owned()],
        Vec::<String>::new(),
        vec!["reset::setup(IOP_PIN_RAW(config::factoryResetButton{{index}}));".to_owned()],
        vec!["reset::resetIfNeeded(loop);".to_owned()],
        vec![],
        vec![NewSensorConfigRequest::new(
            "Button".to_owned(),
            "factoryResetButton{{index}}".to_owned(),
            "Pin".to_owned(),
            NewSensorWidgetKind::PinSelection,
        )],
    )
    .await
}

async fn cooler(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    SensorPrototype::new(
        txn,
        "Cooler",
        Some("cooler"),
        vec![Dependency::new("https://github.com/internet-of-plants/cooler", "main")],
        vec!["cooler.hpp".to_owned()],
        vec![Definition::new("static relay::Cooler cooler{{index}}(IOP_PIN_RAW(config::cooler{{index}}), std::ref({{dhtSensor}}), config::coolerMax{{index}});".to_owned(), vec![SensorReference::new("DHT".to_owned(), "dhtSensor".to_owned())])],
        vec!["cooler{{index}}.begin();\n".to_owned()],
        vec!["cooler{{index}}.actIfNeeded();".to_owned()],
        vec![],
        vec![
            NewSensorConfigRequest::new(
                "Port".to_owned(),
                "cooler{{index}}".to_owned(),
                "Pin".to_owned(),
                NewSensorWidgetKind::PinSelection,
            ),
            NewSensorConfigRequest::new(
                "Max Celsius".to_owned(),
                "coolerMax{{index}}".to_owned(),
                "float".to_owned(),
                NewSensorWidgetKind::F32,
            ),
            NewSensorConfigRequest::new(
                "DHT Sensor".to_owned(),
                "dhtSensor".to_owned(),
                None,
                NewSensorWidgetKind::Sensor("DHT".to_owned()),
            ),
        ],
    )
    .await
}

async fn light_relay(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    SensorPrototype::new(
        txn,
        "Light Relay".to_owned(),
        "light",
        vec![Dependency::new(
            "https://github.com/internet-of-plants/light",
            "main",
        )],
        vec!["light.hpp".to_owned()],
        vec!["static relay::Light light{{index}}(IOP_PIN_RAW(config::light{{index}}));".to_owned()],
        vec![concat!(
            "light{{index}}.begin();\n",
            "for (const auto &[moment, state]: config::lightActions{{index}}) {\n",
            "  light{{index}}.setTime(moment, state);\n",
            "}"
        )
        .to_owned()],
        vec!["light{{index}}.actIfNeeded();".to_owned()],
        vec![],
        vec![
            NewSensorConfigRequest::new(
                "Port".to_owned(),
                "light{{index}}".to_owned(),
                "Pin".to_owned(),
                NewSensorWidgetKind::PinSelection,
            ),
            NewSensorConfigRequest::new(
                "Timed Switches".to_owned(),
                "lightActions{{index}}[]".to_owned(),
                "std::pair<relay::Moment, relay::State>".to_owned(),
                NewSensorWidgetKind::Map(
                    Box::new(NewSensorWidgetKind::Moment),
                    Box::new(NewSensorWidgetKind::Selection(vec![
                        "relay::State::ON".to_owned(),
                        "relay::State::OFF".to_owned(),
                    ])),
                ),
            ),
        ],
    )
    .await
}

async fn water_pump(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    SensorPrototype::new(
        txn,
        "Water Pump".to_owned(),
        "waterPump",
        vec![Dependency::new(
            "https://github.com/internet-of-plants/water-pump",
            "main",
        )],
        vec!["water_pump.hpp".to_owned()],
        vec![
            "static relay::WaterPump waterPump{{index}}(IOP_PIN_RAW(config::waterPump{{index}}));"
                .to_owned(),
        ],
        vec![concat!(
            "waterPump{{index}}.begin();\n",
            "for (const auto &[moment, seconds]: config::waterPumpActions{{index}}) {\n",
            "  waterPump{{index}}.setTime(moment, seconds);\n",
            "}"
        )
        .to_owned()],
        vec!["waterPump{{index}}.actIfNeeded();".to_owned()],
        vec![],
        vec![
            NewSensorConfigRequest::new(
                "Port".to_owned(),
                "waterPump{{index}}".to_owned(),
                "Pin".to_owned(),
                NewSensorWidgetKind::PinSelection,
            ),
            NewSensorConfigRequest::new(
                "Timed Action".to_owned(),
                "waterPumpActions{{index}}[]".to_owned(),
                "std::pair<relay::Moment, iop::time::seconds>".to_owned(),
                NewSensorWidgetKind::Map(
                    Box::new(NewSensorWidgetKind::Moment),
                    Box::new(NewSensorWidgetKind::Seconds),
                ),
            ),
        ],
    )
    .await
}

async fn soil_resistivity(txn: &mut Transaction<'_>) -> Result<SensorPrototype> {
    SensorPrototype::new(
        txn,
        "Soil Resistivity".to_owned(),
        "soilResistivity",
        vec![Dependency::new("https://github.com/internet-of-plants/soil-resistivity", "main")],
        vec!["soil_resistivity.hpp".to_owned()],
        vec![
            "static sensor::SoilResistivity soilResistivity{{index}}(IOP_PIN_RAW(config::soilResistivityPower{{index}}));".to_owned(),
        ],
        vec![
            "soilResistivity{{index}}.begin();".to_owned(),
        ],
        vec![

        ],
        vec![
            SensorMeasurement {
                human_name: "Soil Resistivity Raw".to_owned(),
                name: "soil_resistivity_raw{{index}}".to_owned(),
                value: "soilResistivity{{index}}.measure();".to_owned(),
                ty: SensorMeasurementType::RawAnalogRead,
                kind: SensorMeasurementKind::SoilMoisture,
            },
        ],
        vec![
            // TODO: we should configure the analog pin here too
            NewSensorConfigRequest::new("Port".to_owned(), "soilResistivityPower{{index}}".to_owned(), "Pin".to_owned(), NewSensorWidgetKind::PinSelection),
        ],
    ).await
}

async fn esp8266_target_prototype(txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
    TargetPrototype::new(
        txn,
        "https://ccadb-public.secure.force.com/mozilla/IncludedCACertificateReportPEMCSV".to_owned(),
        "esp8266".to_owned(),
        "-D IOP_ESP8266\n    -D IOP_SSL".to_owned(),
        "espressif8266".to_owned(),
        Some("arduino".to_owned()),
        Some("framework-arduinoespressif8266 @ https://github.com/esp8266/Arduino".to_owned()),
        Some("monitor_filters = esp8266_exception_decoder\nboard_build.f_cpu = 160000000L\nmonitor_speed = 115200".to_owned()),
        Some("deep".to_owned()),
        vec![Dependency::new("https://github.com/esp8266/Arduino", "master"), Dependency::new("https://github.com/platformio/platform-espressif8266", "master")]
    ).await
}

async fn nodemcuv2_esp8266_target(
    txn: &mut Transaction<'_>,
    target_prototype: &TargetPrototype,
) -> Result<Target> {
    Target::new(
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
        vec![
            NewDeviceConfigRequest::new(
                "Captive Portal SSID".to_owned(),
                "SSID".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::SSID,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Captive Portal Password".to_owned(),
                "PSK".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::PSK,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Timezone".to_owned(),
                "timezone".to_owned(),
                "int8_t".to_owned(),
                DeviceWidgetKind::Timezone,
                None,
            ),
        ],
    )
    .await
}

async fn esp32_target_prototype(txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
    let mut prototype = TargetPrototype::new(
        txn,
        "https://hg.mozilla.org/releases/mozilla-release/raw-file/default/security/nss/lib/ckfw/builtins/certdata.txt".to_owned(),
        "esp32".to_owned(),
        "-D IOP_ESP32\n    -D IOP_SSL".to_owned(),
        "https://github.com/internet-of-plants/platform-espressif32".to_owned(),
        Some("arduino".to_owned()),
        Some("toolchain-xtensa-esp32 @ 8.4.0+2021r1
    espressif/toolchain-riscv32-esp @ 8.4.0+2021r1
    framework-arduinoespressif32 @ https://github.com/internet-of-plants/arduino-esp32
    platformio/tool-esptoolpy @ https://github.com/tasmota/esptool/releases/download/v3.2/esptool-v3.2.zip".to_owned()),
    Some("board_build.mcu = esp32\nboard_build.f_cpu = 240000000L\nmonitor_speed = 115200".to_owned()),
        Some("deep".to_owned()),
        vec![Dependency::new("https://github.com/internet-of-plants/arduino-esp32", "master"), Dependency::new("https://github.com/internet-of-plants/platform-espressif32", "master")]
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
    Target::new(
        txn,
        Some("esp32dev".to_owned()),
        vec![],
        "".to_owned(),
        target_prototype,
        vec![
            NewDeviceConfigRequest::new(
                "Captive Portal SSID".to_owned(),
                "SSID".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::SSID,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Captive Portal Password".to_owned(),
                "PSK".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::PSK,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Timezone".to_owned(),
                "timezone".to_owned(),
                "int8_t".to_owned(),
                DeviceWidgetKind::Timezone,
                None,
            ),
        ],
    )
    .await
}

async fn linux_target_prototype(txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
    TargetPrototype::new(
        txn,
        "https://hg.mozilla.org/releases/mozilla-release/raw-file/default/security/nss/lib/ckfw/builtins/certdata.txt".to_owned(),
        "linux".to_owned(),
        "-Wextra
    -Wpedantic
    -pedantic-errors
    -fstack-protector
    -Wstack-protector

    -D IOP_SSL
    -lpthread
    -lcrypto
    -lssl"
            .to_owned(),
        "native".to_owned(),
        None,
        None,
        None,
        None,Vec::new(),
    )
    .await
}

async fn native_linux_target(
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

/// Dummy pin mapping for LINUX mock
enum class Pin { D1 = 5, D2 = 4, D5 = 14, D6 = 12, D7 = 13 };

#endif"
            .to_owned(),
        target_prototype,
        vec![
            NewDeviceConfigRequest::new(
                "Captive Portal SSID".to_owned(),
                "SSID".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::SSID,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Captive Portal Password".to_owned(),
                "PSK".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::PSK,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Timezone".to_owned(),
                "timezone".to_owned(),
                "int8_t".to_owned(),
                DeviceWidgetKind::Timezone,
                None,
            ),
        ],
    )
    .await?;
    target
        .set_build_flags(txn, Some("-D IOP_LINUX".to_owned()))
        .await?;
    Ok(target)
}

async fn native_linux_local_debugging_target(
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

/// Dummy pin mapping for LINUX mock
enum class Pin { D1 = 5, D2 = 4, D5 = 14, D6 = 12, D7 = 13 };

#endif"
            .to_owned(),
        target_prototype,
        vec![
            NewDeviceConfigRequest::new(
                "Captive Portal SSID".to_owned(),
                "SSID".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::SSID,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Captive Portal Password".to_owned(),
                "PSK".to_owned(),
                "iop::StaticString".to_owned(),
                DeviceWidgetKind::PSK,
                Some(SecretAlgo::LibsodiumSealedBox),
            ),
            NewDeviceConfigRequest::new(
                "Timezone".to_owned(),
                "timezone".to_owned(),
                "int8_t".to_owned(),
                DeviceWidgetKind::Timezone,
                None,
            ),
        ],
    )
    .await?;
    target.set_name(txn, Some("mock".to_owned())).await?;
    target
        .set_build_flags(txn, Some("-D IOP_LINUX_MOCK".to_owned()))
        .await?;
    Ok(target)
}
