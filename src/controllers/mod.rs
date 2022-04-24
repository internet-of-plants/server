pub mod collection;
pub mod device;
pub mod device_log;
pub mod device_panic;
pub mod event;
pub mod organization;
pub mod sensor;
pub mod sensor_prototype;
pub mod target;
pub mod target_prototype;
pub mod compiler;
pub mod update;
pub mod user;

pub type Result<T, E = warp::Rejection> = std::result::Result<T, E>;
