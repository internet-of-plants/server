use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use derive_more::{Display, From};
use log::{error, warn};
use serde_json::json;

pub type Result<T, E = Error> = std::result::Result<T, E>;

// TODO: improve error code returned, send error message in debug builds

#[derive(From, Display, Debug)]
pub enum Error {
    Sqlx(sqlx::error::Error),
    Bcrypt(bcrypt::BcryptError),
    Join(tokio::task::JoinError),
    Json(serde_json::Error),
    IO(std::io::Error),
    Fmt(std::fmt::Error),
    Utf8(std::str::Utf8Error),
    Handlebars(handlebars::RenderError),
    EventMustBeObject,
    MeasurementMissing,
    #[display(fmt = "value:{} range:{}", "_0", "_1")]
    MeasurementOutOfRange(String, String),
    #[display(fmt = "type:{:?} expected:{}", "_0", "_1")]
    InvalidMeasurementType(serde_json::Value, String),
    MissingMeasurement(String),
    DuplicatedConfig,
    Unauthorized,
    BadData,
    InsecurePassword,
    InvalidName,
    NoBinaryAvailable,
    NoUpdateAvailable,
    AskedForTooMany,
    CorruptedBinary,
    MissingBinary,
    NothingFound,
    ParseInt(std::num::ParseIntError),
    MissingHeader(&'static str),
    Hyper(hyper::Error),
    InvalidHeaderValue(axum::http::header::InvalidHeaderValue),
    Http(axum::http::Error),
    Multipart(axum::extract::multipart::MultipartError),
}

impl std::error::Error for Error {}

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
            Self::MeasurementOutOfRange(val, range)=> {
                warn!("Measurement Out of Range: {val} expected {range}");
                (StatusCode::BAD_REQUEST, "Measurement Out of Range")
            }
            Self::InvalidMeasurementType(json, ty)=> {
                warn!("Invalid Measurement Type: {json:?} expected {ty}");
                (StatusCode::BAD_REQUEST, "Invalid Measurement Type")
            }
            Self::DuplicatedConfig => {
                warn!("Duplicated Config");
                (StatusCode::BAD_REQUEST, "Duplicated Config")
            }
            Self::MeasurementMissing=> {
                warn!("Measurement Missing");
                (StatusCode::BAD_REQUEST, "Measurement Missing")
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
