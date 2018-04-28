use hex::ToHex;
use sodiumoxide::crypto::hash;
use rand::RngCore;
use rand::os::OsRng;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use std::sync::RwLock;

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

/*

/// Obtains GET params from URL
pub fn get_url_param<'a>(req: &mut Request, param: &str) -> Option<String>{
    match req.extensions.get::<Router>().unwrap().find(param) {
        Some(value) => Some(value.to_owned()),
        None => None
    }
}

/// Returns None if param doesn't exist or is empty, or enum to match data
pub fn get_param(req: &mut Request, key: &str) -> Option<Value> {
    let map = req.get_ref::<Params>().unwrap();
    match map.find(&[key]) {
        Some(&Value::String(ref value)) => if value.chars().count() == 0 {
            None
        } else {
            Some(Value::String(value.clone()))
        },
        Some(&Value::Boolean(value)) => Some(Value::Boolean(value)),
        Some(&Value::I64(value)) => Some(Value::I64(value)),
        Some(&Value::U64(value)) => Some(Value::U64(value)),
        Some(&Value::F64(value)) => Some(Value::F64(value)),
        Some(_) => Some(Value::Null),
        None => None
    }
}

/// Replay all transactions to obtain final balance
pub fn replay_transactions<T: TransactionDbPort>(transactions: LinkedList<T>) -> i64 {
   let mut balance = 0;

    for transaction in transactions.iter() {
        balance += transaction.get_value();
    }
    balance
}

/// Properly format host from config
pub fn get_host() -> String {
    if get_config::<i64>("PORT") == 80 {
        get_config::<String>("HOST")
    } else {
        format!("{}:{}",
                get_config::<String>("HOST"),
                get_config::<i64>("PORT"))

    }
}

/// Turn money received as string to cents for safe manipulation
pub fn to_cents(money: &str) -> i64 {
    let normalized = money.replace(",", ".");
    let pair: Vec<&str> = normalized.split(".")
                                    .collect();
    let decimal = pair[0];
    let mut cents = "00".to_owned();

    if pair.len() > 1 {
        cents = pair[1].to_owned();
        if cents.len() == 1 {
            cents = format!("{}0", cents);
        } else if cents.len() > 2 {
            cents = cents[..2].to_owned();
        }
    }

    let string = format!("{}{}", decimal, cents);

    match string.parse::<i64>() {
        Ok(v) => v,
        Err(err) => {
            println!("{}", err.to_string());
            0
        }
    }
}

/// Turn cents back to displayable money
pub fn from_cents(string: &str) -> String {
    let mut normalized = string;

    let mut decimal = "0";
    let mut cents = "00".to_owned();

    let mut negative = if normalized.chars().nth(0) == Some('-') {
        normalized = &normalized[1..];
        "-"
    } else {
        ""
    };

    if normalized.len() > 2 {
        decimal = &normalized[..normalized.len() - 2];
        cents = normalized[normalized.len() - 2..].to_owned();
    } else if normalized.len() == 2 {
        cents = normalized.to_owned();
    } else if normalized.len() == 1 {
        cents = format!("0{}", normalized)
    }

    if cents == "00" && decimal == "0" {
        negative = "";
    }

    format!("{}{}.{}", negative, decimal, cents)
}

#[cfg(test)]
/// Add csrf_token to requests body
pub fn format_body(body: &str) -> String {
    use lib::auth::generate_csrf_token;

    let separator = if body == "" {
        ""
    } else {
        "&"
    };

    format!("{}{}csrf-token={}",
            body,
            separator,
            generate_csrf_token())
}

#[cfg(test)]
/// Properly format URI from config
pub fn format_url(endpoint: &str) -> String {
    format!("http://{}/{}", get_host(), endpoint)
}

#[cfg(test)]
use iron::Headers;

#[cfg(test)]
/// Get cookie from response and set to request headers
pub fn format_cookies(headers: Headers) -> Vec<String> {
    use iron::headers::SetCookie;
    use std::ops::DerefMut;

    headers.get::<SetCookie>().unwrap()
        .clone()
        .deref_mut()
        .iter_mut()
        .map(|text| text.to_string()
                        .split(" ")
                        .nth(0)
                        .unwrap()
                        .to_owned()
                        .replace(";", ""))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assure money handling is working
    fn money() {
        assert_eq!(from_cents("5000"), "50.00");
        assert_eq!(from_cents("500"), "5.00");
        assert_eq!(from_cents("50"), "0.50");
        assert_eq!(from_cents("5"), "0.05");
        assert_eq!(from_cents("2332"), "23.32");
        assert_eq!(from_cents("-5000"), "-50.00");
        assert_eq!(from_cents("-50"), "-0.50");
        assert_eq!(from_cents("-5"), "-0.05");
        assert_eq!(from_cents("0"), "0.00");
        assert_eq!(from_cents("-0"), "0.00");

        assert_eq!(to_cents("10,3"), 1030);
        assert_eq!(to_cents("10.3333333333333"), 1033);
        assert_eq!(to_cents("20"), 2000);
        assert_eq!(to_cents("-10.02"), -1002);
        assert_eq!(to_cents("0"), 0);
        assert_eq!(to_cents("0.0"), 0);
        assert_eq!(to_cents("-0.00"), 0);
    }
}
*/
