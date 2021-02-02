pub mod device_log;
pub mod device_panic;
pub mod event;
pub mod plant;
pub mod update;
pub mod user;

pub type Result<T, E = warp::Rejection> = std::result::Result<T, E>;
