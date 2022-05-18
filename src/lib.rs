pub mod controllers;
pub mod db;
pub mod error;
pub mod extractor;
pub mod test_helpers;
pub mod utils;

pub use crate::db::{
    collection::{Collection, CollectionId, CollectionView},
    device::{Device, DeviceId, DeviceView, NewDevice},
    device_log::{DeviceLog, DeviceLogId},
    device_panic::{DevicePanic, DevicePanicId},
    event::{Event, EventId, NewEvent},
    organization::{Organization, OrganizationId, OrganizationView},
    update::{Update, UpdateId},
    user::{NewUser, User, UserId, Username},
};

pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::{controllers, db, utils};
    pub use axum::response::IntoResponse;
    #[allow(unused_imports)]
    pub use log::{debug, error, info, trace, warn};
    pub use sqlx::prelude::*;
    pub use tokio::io::AsyncWriteExt;

    pub type Pool = sqlx::PgPool;
    pub type Transaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;

    use crate::db::device::DeviceId;
    use crate::db::user::UserId;
    use serde::Serialize;

    #[derive(sqlx::FromRow, Debug, Clone, PartialEq, Serialize)]
    pub struct Auth {
        pub user_id: UserId,
        pub device_id: Option<DeviceId>,
    }
}

use crate::prelude::*;
use axum::{
    extract::Extension,
    http::{header::HeaderName, header::HeaderValue, Method},
    routing::{delete, get, post},
    Router,
};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use tower_http::cors::{CorsLayer, Origin};
use tracing_subscriber::{prelude::*, EnvFilter};

static INITIALIZED: AtomicBool = AtomicBool::new(false);
pub async fn test_router() -> Router {
    static LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));
    let _guard = LOCK.lock().unwrap();
    if !INITIALIZED.swap(true, Ordering::Relaxed) {
        let url = "postgres://postgres:postgres@127.0.0.1:5432";
        let mut connection = sqlx::PgConnection::connect(url).await.unwrap();
        let _ = sqlx::query("DROP DATABASE iop_test")
            .execute(&mut connection)
            .await;
        sqlx::query("CREATE DATABASE iop_test")
            .execute(&mut connection)
            .await
            .unwrap();
    }
    let url = "postgres://postgres:postgres@127.0.0.1:5432/iop_test";
    router(url).await
}

pub async fn router(url: &str) -> Router {
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| {
                "server=trace,tracing=trace,hyper=info,axum=trace,event=trace,now=trace,timer=trace"
                    .into()
            },
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!(
        "RUST_LOG is {}",
        std::env::var("RUST_LOG").ok().unwrap_or_else(String::new)
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
        HeaderValue::from_static("https://internet-of-plants.github.io"),
        HeaderValue::from_static("https://iop-monitor-server.tk:4001"),
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

    utils::run_migrations(url).await;

    let pool = Pool::connect(url)
        .await
        .expect("Unable to connect to database");
    let pool: &'static Pool = Box::leak(pool.into());

    let app = Router::new()
        .route("/v1/user/login", post(controllers::user::login))
        .route("/v1/user", post(controllers::user::new))
        .route("/v1/sensor", post(controllers::sensor::new))
        .route(
            "/v1/sensors/of/prototype/:id",
            get(controllers::sensor::list_for_prototype),
        )
        .route("/v1/sensors", get(controllers::sensor::list))
        .route(
            "/v1/sensor/prototype/:id",
            get(controllers::sensor_prototype::find),
        )
        .route(
            "/v1/sensor/prototypes",
            get(controllers::sensor_prototype::index),
        )
        .route("/v1/event", post(controllers::event::new))
        .route("/v1/targets", get(controllers::target::list))
        .route("/v1/target", post(controllers::target::new))
        .route(
            "/v1/targets/of/prototype/:id",
            get(controllers::target::list_for_prototype),
        )
        .route(
            "/v1/target/prototypes",
            get(controllers::target_prototype::index),
        )
        .route(
            "/v1/target/prototype/:id",
            get(controllers::target_prototype::find),
        )
        .route("/v1/compiler", post(controllers::compiler::new))
        .route("/v1/compilations", get(controllers::compiler::compilations))
        .route(
            "/v1/compile/:id",
            post(controllers::compiler::compile_firmware),
        )
        .route(
            "/v1/organizations",
            get(controllers::organization::from_user),
        )
        .route("/v1/organization/:id", get(controllers::organization::find))
        .route(
            "/v1/organization/:id/collection/:id",
            get(controllers::collection::find),
        )
        .route(
            "/v1/organization/:id/collection/:id/device/:id",
            get(controllers::device::find),
        )
        .route("/v1/log", post(controllers::device_log::new)) //.and(warp::body::content_length_limit(2048))
        .route(
            "/v1/organization/:id/collection/:id/device/:id/log/last/:limit",
            get(controllers::device_log::index),
        )
        .route("/v1/panic", post(controllers::device_panic::new))
        .route(
            "/v1/organization/:id/collection/:id/device/:id/panic/last/:limit",
            get(controllers::device_panic::index),
        )
        .route("/v1/panic/:id", delete(controllers::device_panic::solve))
        .route(
            "/v1/organization/:id/collection/:id/device/:id/update",
            post(controllers::update::new), //.and(warp::filters::multipart::form().max_length(8 * 1024 * 1024))
        )
        .route(
            "/v1/update",
            get(controllers::update::get), //.and(warp::filters::multipart::form().max_length(8 * 1024 * 1024))
        )
        .layer(Extension(pool))
        .layer(cors);
    //.route("/v1/updates", get(controllers::update::index))
    //.route("/v1/plant/index", get(controllers::plant::index))
    //.route("/v1/plant/history", get(controllers::plant::history))
    //.route("/v1/plant", get(controllers::plant::get))
    app
}