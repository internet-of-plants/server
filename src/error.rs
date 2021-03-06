use derive_more::{Display, From};
use http::StatusCode;
use log::{error, warn};
use std::convert::Infallible;
use warp::Rejection;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(From, Display, Debug)]
pub enum Error {
    Sqlx(sqlx::error::Error),
    Bcrypt(bcrypt::BcryptError),
    IO(std::io::Error),
    Fmt(std::fmt::Error),
    Forbidden,
    BadData,
    NothingFound,
    Warp(warp::Error),
}

impl Error {
    pub async fn handle(rejection: Rejection) -> Result<impl warp::Reply, Infallible> {
        let status = if let Some(error) = rejection.find::<Self>() {
            match error {
                error @ Self::Sqlx(_)
                | error @ Self::Bcrypt(_)
                | error @ Self::Warp(_)
                | error @ Self::IO(_)
                | error @ Self::Fmt(_) => {
                    error!("{:?} {}", error, error);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                Self::Forbidden => {
                    warn!("Forbidden");
                    StatusCode::FORBIDDEN
                }
                Self::BadData => {
                    warn!("Bad Data");
                    StatusCode::BAD_REQUEST
                }
                Self::NothingFound => {
                    warn!("Nothing Found");
                    StatusCode::NOT_FOUND
                }
            }
        } else {
            error!("Unknown internal server error: {:?}", rejection);
            StatusCode::INTERNAL_SERVER_ERROR
        };
        Ok(warp::reply::with_status(status, status))
    }
}

impl std::error::Error for Error {}
impl warp::reject::Reject for Error {}
