use std::convert::Infallible;
use derive_more::{Display, From};
use log::{error, warn};
use http::StatusCode;
use warp::Rejection;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(From, Display, Debug)]
pub enum Error {
    Sqlx(sqlx::error::Error),
    Bcrypt(bcrypt::BcryptError),
    Forbidden,
    BadData,
    NothingFound,
}

impl Error {
    pub async fn handle(rejection: Rejection) -> Result<impl warp::Reply, Infallible> {
        let status = if let Some(error) = rejection.find::<Self>() {
            match error {
                error @ Self::Sqlx(_) | error @ Self::Bcrypt(_) => {
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

impl From<Error> for Rejection {
    fn from(err: Error) -> Self {
        warp::reject::custom(err)
    }
}
