pub mod plant;
pub mod event;
pub mod user;
pub mod error;

pub type Result<T, E = warp::Rejection> = std::result::Result<T, E>;
