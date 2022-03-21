pub mod collection;
pub mod device;
pub mod device_log;
pub mod device_panic;
pub mod event;
pub mod update;
pub mod user;
pub mod organization;

pub type Result<T, E = warp::Rejection> = std::result::Result<T, E>;
