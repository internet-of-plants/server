use crate::CompilerId;
use axum::response::{IntoResponse, Response};
use axum::{http::StatusCode, Json};
use backtrace::Backtrace;
use log::{error, warn};
use serde_json::json;
use std::collections::HashSet;

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
    NewSensorReferencedDoesntExist(usize, HashSet<usize>),
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
