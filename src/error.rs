use axum::http::StatusCode;
use derive_more::{Display, From};
use log::{error, warn};
use serde_json::json;
//use std::convert::Infallible;
use axum::response::{IntoResponse, Response};
use axum::Json;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(From, Display, Debug)]
pub enum Error {
    Sqlx(sqlx::error::Error),
    Bcrypt(bcrypt::BcryptError),
    Join(tokio::task::JoinError),
    IO(std::io::Error),
    Fmt(std::fmt::Error),
    Utf8(std::str::Utf8Error),
    Forbidden,
    BadData,
    NotModified,
    NothingFound,
    CorruptBinary,
    ParseInt(std::num::ParseIntError),
    MissingHeader(&'static str),
    Hyper(hyper::Error),
    InvalidHeaderValue(axum::http::header::InvalidHeaderValue),
    Http(axum::http::Error)
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::Http(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Hyper(error) => {
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
            Self::InvalidHeaderValue(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::ParseInt(error) => {
                error!("{:?} {}", error, error);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::Forbidden => {
                warn!("Forbidden");
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
            Self::CorruptBinary => {
                warn!("Corrupt Binary");
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            }
            Self::NotModified => (StatusCode::NOT_MODIFIED, "Not modified"),
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
