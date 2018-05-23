use actix_web::{HttpResponse, ResponseError};
use base64::DecodeError;
use diesel::result::{DatabaseErrorKind::ForeignKeyViolation, DatabaseErrorKind::UniqueViolation,
                     Error as DieselError};
use hex::FromHexError;
use image::ImageError;
use r2d2::Error as R2d2Error;
use rand::Error as RandError;
use std::{error::Error as StdError, fmt::Display, fmt::Formatter, fmt::Result as FmtResult};

use LOG;

#[derive(Debug)]
pub enum Error {
    Diesel(DieselError),
    R2d2(R2d2Error),
    SodiumOxide(()),
    Hex(FromHexError),
    Rand(RandError),
    Image(ImageError),
    Base64(DecodeError),
    NotAuthenticated,
    InvalidCredentials,
    InvalidData,
}

impl_err! {
    Error;
    Diesel(DieselError),
    SodiumOxide(()),
    Hex(FromHexError),
    Rand(RandError),
    R2d2(R2d2Error),
    Image(ImageError),
    Base64(DecodeError),
}

impl Error {
    fn response_type(&self) -> ResponseType {
        use self::Error::*;
        error!(LOG.read().unwrap(), "{:?}", self);
        match self {
            Diesel(DieselError::NotFound) => ResponseType::NotFound,
            Diesel(DieselError::DatabaseError(UniqueViolation, _)) => ResponseType::NonUnique,
            Diesel(DieselError::DatabaseError(ForeignKeyViolation, _)) => ResponseType::BadRequest,
            Diesel(_) => ResponseType::InternalServerError,
            R2d2(_) => ResponseType::InternalServerError,
            Hex(_) => ResponseType::InternalServerError,
            Rand(_) => ResponseType::InternalServerError,
            Image(_) => ResponseType::BadRequest,
            SodiumOxide(()) => ResponseType::InternalServerError,
            Base64(_) => ResponseType::BadRequest,
            NotAuthenticated => ResponseType::Unauthorized,
            InvalidCredentials => ResponseType::InvalidCredentials,
            InvalidData => ResponseType::BadRequest,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Error")
    }
}

impl StdError for Error {
    fn description(&self) -> &'static str {
        "Application Error"
    }
}

#[derive(Debug)]
pub enum ResponseType {
    InternalServerError,
    NotFound,
    Unauthorized,
    InvalidCredentials,
    BadRequest,
    NonUnique,
}

impl Display for ResponseType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "ResponseType")
    }
}

impl StdError for ResponseType {
    fn description(&self) -> &'static str {
        "Response Error"
    }
}

impl ResponseError for ResponseType {
    fn error_response(&self) -> HttpResponse {
        use self::ResponseType::*;
        match self {
            InternalServerError => HttpResponse::InternalServerError().finish(),
            NotFound => HttpResponse::NotFound().finish(),
            Unauthorized => HttpResponse::Unauthorized().finish(),
            InvalidCredentials => HttpResponse::Unauthorized().finish(),
            BadRequest => HttpResponse::BadRequest().finish(),
            NonUnique => HttpResponse::Conflict().finish(),
        }
    }
}
