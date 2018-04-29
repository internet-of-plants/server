use hex::ToHex;
use sodiumoxide::crypto::hash;
use rand::RngCore;
use rand::os::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::RwLock;

pub type UID = i32;
pub type BigUID = i64;
pub type Celsius = i16;
pub type Percentage = i16;
pub type AnalogRead = i16;

lazy_static! {
    pub static ref REVERSE_ROUTE_TABLE: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

/// Returns the number of seconds since UNIX_EPOCH (1970)
pub fn unix_epoch() -> u64 {
    // Returns 0 if now() is before UNIX_EPOCH (1970)
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_secs(),
        Err(_) => 0
    }
}

/// Returns the hash of argument
pub fn basic_hash(src: &str) -> String {
    hash::hash(src.as_bytes()).as_ref().to_hex()
}

/// Returns randomly generated string with specified size
pub fn random_string(len: usize) -> String {
    let mut rng = OsRng::new().unwrap();
    let mut hash = String::new();
    while hash.len() < len {
        hash.push_str(&basic_hash(&rng.next_u32().to_string()));
    }

    hash[..len].to_owned()
}
