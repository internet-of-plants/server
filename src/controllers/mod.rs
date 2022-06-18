pub mod collection;
pub mod compiler;
pub mod device;
pub mod device_log;
pub mod device_panic;
pub mod event;
pub mod firmware;
pub mod organization;
pub mod sensor;
pub mod sensor_prototype;
pub mod target;
pub mod target_prototype;
pub mod update;
pub mod user;

pub type Result<T, E = crate::error::Error> = std::result::Result<T, E>;
