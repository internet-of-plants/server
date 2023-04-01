pub mod controllers;
pub mod db;
pub mod error;
pub mod extractor;
pub mod test_helpers;
pub mod utils;

pub use crate::db::{
    auth::{Auth, AuthToken},
    collection::{Collection, CollectionId, CollectionView},
    compilation::{Compilation, CompilationId, CompilationView},
    compiler::{Compiler, CompilerId, CompilerView, NewCompiler},
    device::{Device, DeviceId, DeviceView, NewDevice},
    device_config::{DeviceConfig, DeviceConfigId, DeviceConfigView, NewDeviceConfig},
    device_config_request::{
        DeviceConfigRequest, DeviceConfigRequestId, DeviceConfigRequestView, NewDeviceConfigRequest,
    },
    device_config_type::{
        DeviceConfigType, DeviceConfigTypeId, DeviceConfigTypeView, DeviceWidgetKind,
    },
    device_log::{DeviceLog, DeviceLogId, DeviceLogView},
    device_panic::{DevicePanic, DevicePanicId, DevicePanicView, NewDevicePanic},
    event::{DeviceStat, Event, EventId, EventView},
    firmware::{Firmware, FirmwareId, FirmwareView},
    organization::{Organization, OrganizationId, OrganizationView},
    secret::SecretAlgo,
    sensor::{
        Definition, Include, NewSensor, Sensor, SensorId, SensorPrototypeDefinitionId,
        SensorReference, SensorView, Setup, UnauthenticatedAction,
    },
    sensor_config::{NewSensorConfig, SensorConfig, SensorConfigId, SensorConfigView, Val, ValRaw},
    sensor_config_request::{
        NewSensorConfigRequest, SensorConfigRequest, SensorConfigRequestId, SensorConfigRequestView,
    },
    sensor_config_type::{
        NewSensorWidgetKind, SensorConfigType, SensorConfigTypeId, SensorConfigTypeMapId,
        SensorConfigTypeView, SensorWidgetKindRaw, SensorWidgetKindView,
    },
    sensor_measurement::{
        SensorMeasurement, SensorMeasurementKind, SensorMeasurementType, SensorMeasurementView,
    },
    sensor_prototype::{SensorPrototype, SensorPrototypeId, SensorPrototypeView},
    target::{Target, TargetId, TargetView},
    target_prototype::{
        Certificate, CertificateId, Dependency, TargetPrototype, TargetPrototypeId,
    },
    user::{Login, NewUser, User, UserId, Username},
};
pub use error::{Error, Result};

pub type DateTime = chrono::DateTime<chrono::Utc>;
pub type Pool = sqlx::PgPool;
pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

pub mod logger {
    pub use tracing::{debug, error, info, trace, warn};
}

use logger::*;

use axum::{
    extract::Extension,
    http::{header::HeaderName, header::HeaderValue, Method},
    routing::{get, post},
    Router,
};
use sqlx::Connection;
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, Origin};
use tracing_subscriber::{prelude::*, EnvFilter};

pub async fn test_router() -> Router {
    static LOCK: Mutex<bool> = Mutex::const_new(false);
    let mut guard = LOCK.lock().await;
    if !std::mem::replace(&mut *guard, true) {
        let url = "postgres://postgres:postgres@127.0.0.1:5432";
        let mut connection = sqlx::PgConnection::connect(url).await.unwrap();
        let _ = sqlx::query("DROP DATABASE iop_test")
            .execute(&mut connection)
            .await;
        sqlx::query("CREATE DATABASE iop_test")
            .execute(&mut connection)
            .await
            .unwrap();
        tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| {
                "server=trace,tracing=trace,hyper=info,axum=trace,event=trace,now=trace,timer=trace"
                    .into()
            },
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();
    }
    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop_test";
    let pool = Pool::connect(url)
        .await
        .expect("Unable to connect to database");
    let pool: &'static Pool = Box::leak(pool.into());
    router(pool).await
}

pub async fn router(pool: &'static Pool) -> Router {
    info!(
        "RUST_LOG is {}",
        std::env::var("RUST_LOG").ok().unwrap_or_default()
    );

    #[cfg(debug_assertions)]
    let allowed_origin = vec![
        HeaderValue::from_static("http://127.0.0.1:8080"),
        HeaderValue::from_static("http://localhost:8080"),
        HeaderValue::from_static("http://127.0.0.1:4001"),
        HeaderValue::from_static("http://localhost:4001"),
    ];

    #[cfg(not(debug_assertions))]
    let allowed_origin = vec![
        HeaderValue::from_static("http://localhost:8080"),
        HeaderValue::from_static("https://web.internet-of-plants.org"),
        HeaderValue::from_static("https://api.internet-of-plants.org:4001"),
    ];

    let cors = CorsLayer::new()
        .allow_credentials(false)
        .allow_headers(vec![
            HeaderName::from_static("authorization"),
            HeaderName::from_static("content-type"),
            HeaderName::from_static("mac_address"),
            HeaderName::from_static("driver"),
            HeaderName::from_static("version"),
            HeaderName::from_static("time_running"),
            HeaderName::from_static("vcc"),
            HeaderName::from_static("free_dram"),
            HeaderName::from_static("free_iram"),
            HeaderName::from_static("free_stack"),
            HeaderName::from_static("biggest_block_dram"),
            HeaderName::from_static("biggest_block_iram"),
            HeaderName::from_static("x-esp8266-sta-mac"),
            HeaderName::from_static("x-esp8266-sketch-md5"),
        ])
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
            Method::PUT,
        ])
        .allow_origin(Origin::list(allowed_origin));

    utils::run_migrations(pool).await;

    Router::new()
        .route("/v1/user/login", post(controllers::user::login))
        .route("/v1/user", post(controllers::user::new))
        .route("/v1/targets", get(controllers::target::list))
        .route(
            "/v1/target/sensor/prototypes",
            get(controllers::sensor_prototype::list_for_target),
        )
        .route("/v1/sensor/alias", post(controllers::sensor::set_alias))
        .route("/v1/sensor/color", post(controllers::sensor::set_color))
        .route("/v1/compiler", post(controllers::compiler::new))
        .route("/v1/compiler/set", post(controllers::compiler::set))
        .route("/v1/compilers", get(controllers::compiler::list))
        .route(
            "/v1/organizations",
            get(controllers::organization::from_user),
        )
        .route("/v1/organization", get(controllers::organization::find))
        .route("/v1/collection", get(controllers::collection::find))
        .route(
            "/v1/collection/name",
            post(controllers::collection::set_name),
        )
        .route("/v1/device", get(controllers::device::find))
        .route("/v1/device/events", get(controllers::event::list))
        .route("/v1/device/logs", get(controllers::device_log::list))
        .route("/v1/device/panics", get(controllers::device_panic::list))
        .route("/v1/device/name", post(controllers::device::set_name))
        .route(
            "/v1/device/panic/solve",
            post(controllers::device_panic::solve),
        )
        .route("/v1/event", post(controllers::event::new))
        .route("/v1/log", post(controllers::device_log::new)) //.and(warp::body::content_length_limit(2048))
        .route("/v1/panic", post(controllers::device_panic::new))
        .route("/v1/update", get(controllers::firmware::update))
        .layer(Extension(pool))
        .layer(cors)
}
