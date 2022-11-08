# Internet of Plants - Central Management Server

This is the central server that monitors and manages iot devices. Integrates with [internet-of-plants/iop](https://github.com/internet-of-plants/iop).

Will collect measurements, logs and panic events from the devices and organize them in groups that make sense, to help managing them in bulk.

It can also be interfaced with [`internet-of-plants/client`](https://github.com/internet-of-plants/client). It displays what is going on with the iot devices in real time and allows you to manage them at scale. While keeping them secure and up to date.

Acts on GET (url encoded data) or POST (json encoded data) HTTP requests, producing the desired data in JSON and/or updating the system accordingly. Each user has their account with their devices. The devices receive constant events from the embedded firmware, so you can keep track of them.

The ideal setup is to configure a target device and a set of sensors, with their specified digital ports and factory settings in this server. With that a compiler will be created.

This compiler will compile this configuration set from the [`internet-of-plants/client`](https://github.com/internet-of-plants/client) and will generate a C++17 code that integrates [`internet-of-plants/iop`](https://github.com/internet-of-plants/iop) and the specified sensor libraries. Reporting itself to this server.

The generated C++ code will be then compiled for the desired target and then be provided as a over-the-air firmware update for that device. Now you can track the measurements metadata and get updates when any dependency updates.

The device can have an empty compiler, providing arbitrary data, although that makes it harder to analyze the measurements as they lack in some metadata needed (like which device generated that measurement).

## Features

### User interface requests

- All routes, except for `/v1/signup` and `/v1/user/login` require an authentication token
    - `Authorization` header set to `Basic ${token}`
- POST `/v1/user`: User signup
    - JSON request: `{ email: string; user: string; password: string }` returns base64 token
- POST `/v1/user/login`: User authentication
    - JSON request: `{ user: string; password: string }` returns base64 token
- GET `/v1/targets`: List all targets available - to be attached as device metadata
- GET `/v1/target/sensors/prototype`: List all sensors to be configured for specified target
    - URL encoded: `targetId=${TargetId}`
- GET `/v1/organizations`
- GET `/v1/organization`
    - URL encoded: `organizationId=${OrganizationId}`
- GET `/v1/collection`
    - URL encoded: `collectionId=${CollectionId}`
- GET `/v1/device`
    - URL encoded: `deviceId=${DeviceId}`
- GET `/v1/device/events`
    - JSON request: `{ deviceId: DeviceId; limit: u32 }`
- GET `/v1/device/logs`
    - JSON request: `{ deviceId: DeviceId; limit: u32 }`
- GET `/v1/device/panics`
    - JSON request: `{ deviceId: DeviceId; limit: u32 }`
- POST `/v1/device/name`
    - JSON request: `{ deviceId: DeviceId; name: string }`
- POST `/v1/device/panic/solve`
    - JSON request: `{ deviceId: DeviceId; panicId: PanicId }`
- POST `/v1/sensor/alias`
    - JSON request: `{ deviceId: DeviceId; sensorId: SensorId; alias: string }`
- POST `/v1/compiler`
    - NewConfig: `{ requestId: ConfigRequestId; value: string }`
        - value is encoded the way it will be used by C++
    - NewSensor: `{ prototypeId: SensorPrototypeId; alias: string; configs: NewConfig[] }`
    - JSON request: `{ deviceId: DeviceId; targetId: TargetId; sensors: NewSensor[] }`

### Device requests

- POST `/v1/user/login`: Device authentication
    - JSON request: `{ user: string; password: string }` returns base64 token
- POST `/v1/event`: Register Device Measurements
    - Arbitrary JSON request, type-checked if a compiler is attached to the device
    - Compilers are made of a target + configured sensors, it generates the C++ code
- POST `/v1/log`: Register Device Logs
    - Lossy UTF8 body containing plain log message
- POST `/v1/panic`: Report Device Panic
    - JSON request: `{ file: string; line: i32; func: string; msg: string }`
    - `MAC_ADDRESS` + `VERSION` (Firmare's MD5 hash) headers
- GET `/v1/update`: Update device firmware update binary if available

## Dependencies

It should work on all GNU/Linux distributions, Windows devices and MacOS devices.

But all production focused devops scripts assume the user is on Ubuntu and has postgres, [rustc + cargo](https://rustup.rs) and [platformio cli](https://docs.platformio.org/en/latest/core/installation.html#installation-methods) installed.

## Setup local environment

*This scripts install postgresql, creates a database named iop and sets 'postgres' psql user's password to 'postgres' (only available at 127.0.0.1)*

`./tools/install-dependencies.sh`

# Deploy

Check [DEPLOY.md](https://github.com/internet-of-plants/blob/main)

## License

[GNU Affero General Public License version 3 or later (AGPL-3.0+)](https://github.com/internet-of-plants/server/blob/main/LICENSE.md)
