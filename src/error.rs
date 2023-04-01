use crate::{CompilerId, NewSensor, SensorPrototypeId, SensorWidgetKindView, ValRaw};
use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use backtrace::Backtrace;
use serde_json::json;
use std::collections::HashSet;
use tracing::{error, warn};

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Sqlx(sqlx::error::Error),
    #[error(transparent)]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Fmt(#[from] std::fmt::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    Handlebars(#[from] handlebars::RenderError),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] axum::http::header::InvalidHeaderValue),
    #[error(transparent)]
    Http(#[from] axum::http::Error),
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Git2(#[from] git2::Error),
    #[error("event must be object")]
    EventMustBeObject,
    #[error("measurement missing")]
    MeasurementMissing,
    #[error("value: {0} range: {1}")]
    MeasurementOutOfRange(String, String),
    #[error("type: {0:?} expected: {1}")]
    InvalidMeasurementType(serde_json::Value, String),
    #[error("missing measurement {0}")]
    MissingMeasurement(String),
    #[error("duplicated config")]
    DuplicatedConfig,
    #[error("duplicated key")]
    DuplicatedKey,
    #[error("unauthorized")]
    Unauthorized,
    #[error("bad data")]
    BadData,
    #[error("insecure password")]
    InsecurePassword,
    #[error("invalid name")]
    InvalidName,
    #[error("no binary available")]
    NoBinaryAvailable,
    #[error("no update available")]
    NoUpdateAvailable,
    #[error("asked for too many")]
    AskedForTooMany,
    #[error("corrupted binary")]
    CorruptedBinary,
    #[error("missing binary")]
    MissingBinary,
    #[error("nothing found")]
    NothingFound,
    #[error("no collection for compiler: {0}")]
    NoCollectionForCompiler(CompilerId),
    #[error("missing header {0}")]
    MissingHeader(&'static str),
    #[error("invalid timezone {1}: {0}")]
    InvalidTimezone(std::num::ParseIntError, String),
    #[error("new sensor referenced by {0} doesnt exist in {1:?}")]
    NewSensorReferencedDoesntExist(u64, HashSet<u64>),
    #[error("no variable name for referenced sensor")]
    NoVariableNameForReferencedSensor(SensorPrototypeId),
    #[error("invalid moment {0:02}:{1:02}:{2:02}")]
    InvalidMoment(u8, u8, u8),
    #[error("integer {0} out of range {1:?}")]
    IntegerOutOfRange(u64, SensorWidgetKindView),
    #[error("float {0} out of range {1:?}")]
    FloatOutOfRange(f64, SensorWidgetKindView),
    #[error("wrong sensor kind got {0} expected {1}")]
    WrongSensorKind(SensorPrototypeId, SensorPrototypeId),
    #[error("invalid {0} selection of {1:?}")]
    InvalidSelection(String, Vec<String>),
    #[error("invalid val for sensor {0:?}")]
    InvalidValForSensor(ValRaw),
    #[error("invalid val type of {0:?} for {1:?}")]
    InvalidValType(ValRaw, SensorWidgetKindView),
    #[error("sensor {0} referenced not found {1:?}")]
    SensorReferencedNotFound(u64, NewSensor),
}

impl From<sqlx::error::Error> for Error {
    fn from(err: sqlx::error::Error) -> Self {
        error!("{err}\n{:?}", Backtrace::new());
        Self::Sqlx(err)
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Multipart(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Http(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Hyper(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Reqwest(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Json(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Sqlx(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Join(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Bcrypt(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::IO(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Fmt(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Utf8(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Handlebars(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::InvalidHeaderValue(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::ParseInt(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Git2(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::NewSensorReferencedDoesntExist(pk, list) => {
                error!("Unable to find sensor {} in {:?}", pk, list);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::NoVariableNameForReferencedSensor(prototype_id) => {
                warn!("No variable name for referenced sensor {prototype_id}");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Unauthorized => {
                warn!("Unauthorized");
                (StatusCode::UNAUTHORIZED, "Unauthorized")
            }
            Self::MissingHeader(header) => {
                error!("Missing HeaderBad: {}", header);
                (StatusCode::BAD_REQUEST, "Bad Request")
            }
            Self::BadData => {
                warn!("Bad Data");
                (StatusCode::BAD_REQUEST, "Bad Request")
            }
            Self::EventMustBeObject => {
                warn!("Event Must Be Object");
                (StatusCode::BAD_REQUEST, "Event Must Be Object")
            }
            Self::MeasurementOutOfRange(val, range) => {
                warn!("Measurement Out of Range: {val} expected {range}");
                (StatusCode::BAD_REQUEST, "Measurement Out of Range")
            }
            Self::InvalidMeasurementType(json, ty) => {
                warn!("Invalid Measurement Type: {json:?} expected {ty}");
                (StatusCode::BAD_REQUEST, "Invalid Measurement Type")
            }
            Self::DuplicatedConfig => {
                warn!("Duplicated Config");
                (StatusCode::BAD_REQUEST, "Duplicated Config")
            }
            Self::DuplicatedKey => {
                warn!("Duplicated Key");
                (StatusCode::BAD_REQUEST, "Duplicated Key")
            }
            Self::MeasurementMissing => {
                warn!("Measurement Missing");
                (StatusCode::BAD_REQUEST, "Measurement Missing")
            }
            Self::NoCollectionForCompiler(id) => {
                warn!("No Collection For Compiler: {id:?}");
                (StatusCode::BAD_REQUEST, "No Collection For Compiler")
            }
            Self::MissingMeasurement(m) => {
                warn!("Measurement Missing: {m}");
                (StatusCode::BAD_REQUEST, "Measurement Missing")
            }
            Self::InsecurePassword => {
                warn!("Insecure Password");
                (StatusCode::BAD_REQUEST, "Invalid Password")
            }
            Self::MissingBinary => {
                warn!("Missing Binary");
                (StatusCode::BAD_REQUEST, "Missing Binary")
            }
            Self::InvalidName => {
                warn!("Invalid Name");
                (StatusCode::BAD_REQUEST, "Invalid Name")
            }
            Self::NoBinaryAvailable => {
                warn!("No Binary Available");
                (StatusCode::BAD_REQUEST, "No Binary Available")
            }
            Self::NoUpdateAvailable => {
                warn!("No Update Available");
                (StatusCode::BAD_REQUEST, "No Update Available")
            }
            Self::CorruptedBinary => {
                warn!("Corrupted Binary");
                (StatusCode::BAD_REQUEST, "Corrupted Binary")
            }
            Self::AskedForTooMany => {
                warn!("Asked For Too Many");
                (StatusCode::BAD_REQUEST, "Asked For Too Many")
            }
            Self::InvalidTimezone(err, tz) => {
                warn!("Invalid Timezone {tz}: {err}");
                (StatusCode::BAD_REQUEST, "Invalid Timezone")
            }
            Self::InvalidMoment(hours, minutes, seconds) => {
                warn!("Invalid moment {hours:02}:{minutes:02}:{seconds:02}");
                (StatusCode::BAD_REQUEST, "Invalid Moment")
            }
            Self::IntegerOutOfRange(num, widget) => {
                warn!("Integer {num} out of range of widget {widget:?}");
                (StatusCode::BAD_REQUEST, "Integer Out Of Range")
            }
            Self::FloatOutOfRange(num, widget) => {
                warn!("Float {num} out of range of widget {widget:?}");
                (StatusCode::BAD_REQUEST, "Invalid Timezone")
            }
            Self::WrongSensorKind(got, expected) => {
                warn!("wrong sensor kind got {got} expected {expected}");
                (StatusCode::BAD_REQUEST, "Invalid Sensor Kind")
            }
            Self::InvalidSelection(selected, selection) => {
                warn!("invalid {selected} selection of {selection:?}");
                (StatusCode::BAD_REQUEST, "Invalid Selection")
            }
            Self::InvalidValForSensor(raw) => {
                warn!("invalid val for sensor {raw:?}");
                (StatusCode::BAD_REQUEST, "Invalid Sensor")
            }
            Self::InvalidValType(raw, widget) => {
                warn!("invalid val type of {raw:?} for {widget:?}");
                (StatusCode::BAD_REQUEST, "Invalid Type")
            }
            Self::SensorReferencedNotFound(pk, sensor) => {
                warn!("sensor {pk} referenced not found {sensor:?}");
                (StatusCode::BAD_REQUEST, "Invalid Sensor")
            }
            Self::NothingFound => {
                warn!("Nothing Found");
                (StatusCode::NOT_FOUND, "Not found")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
