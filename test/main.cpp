#include <iop/loop.hpp>
#include <pin.hpp>
#include <factory_reset_button.hpp>
#include <soil_resistivity.hpp>
#include <dallas_temperature.hpp>
#include <dht.hpp>

namespace config {
constexpr static iop::time::milliseconds measurementsInterval = 180 * 1000;
constexpr static Pin factoryResetButton = Pin::D1;
constexpr static Pin soilResistivityPower = Pin::D7;
constexpr static Pin soilTemperature = Pin::D5;
constexpr static Pin airTempAndHumidity = Pin::D6;
constexpr static dht::Version dhtVersion = dht::Version::DHT22;
}

static sensor::SoilResistivity soilResistivity(IOP_PIN_RAW(config::soilResistivityPower));
static dallas::TemperatureCollection soilTemperature(IOP_PIN_RAW(config::soilTemperature));
static dht::Dht airTempAndHumidity(IOP_PIN_RAW(config::airTempAndHumidity), config::dhtVersion);

auto reportMeasurements(iop::EventLoop &loop, const iop::AuthToken &token) noexcept -> void {
  loop.logger().debug(IOP_STR("Handle Measurements"));

  const auto json = loop.api().makeJson(IOP_FUNC, [](JsonDocument &doc) {
    doc["soil_resistivity_raw"] = soilResistivity.measure();
    doc["soil_temperature_celsius"] = soilTemperature.measure();
    doc["air_temperature_celsius"] = airTempAndHumidity.measureTemperature();
    doc["air_humidity_percentage"] = airTempAndHumidity.measureHumidity();
    doc["air_heat_index_celsius"] = airTempAndHumidity.measureHeatIndex();
  });
  if (!json) iop_panic(IOP_STR("Unable to send measurements, buffer overflow"));

  loop.registerEvent(token, *json);
}

namespace iop {
auto setup(EventLoop &loop) noexcept -> void {
  reset::setup(IOP_PIN_RAW(config::factoryResetButton));
  loop.setInterval(1000, reset::resetIfNeeded);
  soilResistivity.begin();
  soilTemperature.begin();
  airTempAndHumidity.begin();
  loop.setAuthenticatedInterval(config::measurementsInterval, reportMeasurements);
}
}
