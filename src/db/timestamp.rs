use serde::{Deserialize, Serialize};
use std::time::SystemTime;

pub type DateTime = chrono::DateTime<chrono::Utc>;

pub fn now() -> DateTime {
    SystemTime::now().into()
}
