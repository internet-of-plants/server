import requests

if __name__ == "__main__":
    mac_address = "AA:BB:CC:DD:EE"
    version = "ABCD"
    time_running = 0
    vcc = "1024"
    free_dram = "1000"
    free_iram = "1000"
    free_stack = "1000"
    biggest_block_dram = "1000"
    biggest_block_dram = "1000"

    air_temperature_celsius = 0
    air_humidity_percentage = 0
    air_heat_index_celsius = 0
    soil_resistivity_raw = 0
    soil_temperature_celsius = 0

    requests.post("https://localhost:3000/event", data = {
        "air_temperature_celsius": air_temperature_celsius,
        "air_humidity_percentage": air_humidity_percentage,
        "air_heat_index_celsius": air_heat_index_celsius,
        "soil_resistivity_raw": soil_resistivity_raw,
        "soil_temperature_celsius": soil_temperature_celsius
    }, headers = {
        "MAC_ADDRESS": mac_address,
        "VERSION": version,
        "TIME_RUNNING": time_running,
        "VCC": vcc,
        "FREE_DRAM": free_dram,
        "FREE_IRAM": free_iram,
        "FREE_STACK": free_stack,
        "BIGGEST_BLOCK_DRAM": biggest_dram_block,
        "BIGGEST_BLOCK_IRAM": biggest_iram_block,
    })
