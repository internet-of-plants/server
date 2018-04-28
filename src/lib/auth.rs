use gotham::state::FromState;
use gotham::middleware::session::SessionData;
use lib::utils::random_string;
use gotham::state::State;

#[derive(StateData, Serialize, Deserialize, Debug)]
pub struct Session {
    csrf_token: String,
    id: Option<i64>
}

#[derive(StateData, Deserialize, Debug)]
struct CsrfToken {
    csrf_token: String
}

pub fn is_auth(state: &State) -> bool {
    false
}

pub fn is_csrf_valid(state: &mut State) -> bool {
    match (__from_body!(state, CsrfToken), get_csrf_token(&state)) {
        (Some(CsrfToken { csrf_token: form }), Some(cookie)) => if *cookie == form {
            return true
        },
        _ => return false
    }
    false
}

pub fn set_csrf_token(state: &mut State) {
    let session = SessionData::<Option<Session>>::borrow_mut_from(state);
    match **session {
        Some(_) => {},
        None => **session = Some(Session { csrf_token: random_string(30), id: None})
    }
}

pub fn get_csrf_token(state: &State) -> Option<&String> {
    match **SessionData::<Option<Session>>::borrow_from(state) {
        Some(Session { ref csrf_token, id }) => Some(csrf_token),
        None => None
    }
}

/*
use lib::hex::{FromHex, ToHex};
use iron_sessionstorage::{Value, SessionRequestExt};
use iron::method::{Get, Post};
use params;
use iron::Request;
use sodiumoxide::crypto::pwhash;
use config::get_config;
use lib::utils::{unix_epoch, basic_hash, get_param, random_string};
use lib::serde_json;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Auth {
    actor_id: i64,
    expiration_date: u64
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct CsrfToken {
    token: String
}

impl Value for CsrfToken {
    fn get_key() -> &'static str { "csrf_token" }
    fn into_raw(self) -> String { serde_json::to_string(&self).unwrap() }
    fn from_raw(value: String) -> Option<CsrfToken>  { serde_json::from_str(&value).unwrap() }
}

impl Value for Auth {
    fn get_key() -> &'static str { "actor" }
    fn into_raw(self) -> String { serde_json::to_string(&self).unwrap() }
    fn from_raw(value: String) -> Option<Auth>  { serde_json::from_str(&value).unwrap() }
}

/// Returns Result with actor_id stored in signed session (if auth, else None)
pub fn get_actor_id(req: &mut Request) -> Option<i64> {
    match req.session().get::<Auth>() {
        Ok(Some(auth)) => Some(auth.actor_id),
        _ => None
    }
}

/// Abstract CSRF generation, make tests easier
pub fn generate_csrf_token() -> String {
    let _token = random_string(30);

    // TODO: Create test end-point to retrieve real CSRF token instead
    //       of bypassing its randomness
    #[cfg(test)]
    // Allows predicting tests values
    return "4".to_string();

    // Avoid warning, during test this is dead-code
    #[cfg(not(test))]
    _token
}

/// Checks for CSRF token, if it's POST request, sets token if missing in GET request
pub fn csrf_auth(req: &mut Request) -> bool {
    match req.method {
        Post => {
            let from_cookie = match req.session().get::<CsrfToken>() {
                Ok(Some(cookie)) => Some(cookie),
                _ => None
            };

            let from_form = match get_param(req, "csrf-token") {
                Some(params::Value::String(value)) => Some(value),
                _ => None
            };

            match (from_form, from_cookie) {
                (Some(form), Some(cookie)) => form == cookie.token,
                _ => false
            }
        },
        Get => {
            match req.session().get::<CsrfToken>() {
                Ok(Some(_)) => {},
                _ => req.session().set(CsrfToken {
                    token: generate_csrf_token()
                }).unwrap(),
            }

            true
        },
        _ => false
    }
}

/// Returns boolean result, checking the session expiration (if exists at all)
/// Checks for CSRF token, if it's POST request
pub fn is_auth(req: &mut Request) -> bool {
    let auth = match req.session().get::<Auth>() {
        Ok(Some(auth)) => unix_epoch() < auth.expiration_date,
        _ => false
    };

    csrf_auth(req) && auth
}

/// Returns CSRF Token stored in cookie
pub fn get_csrf_token(req: &mut Request) -> Option<String> {
    match req.session().get::<CsrfToken>() {
        Ok(Some(cookie)) => Some(cookie.token),
        _ => None
    }
}

/// Stores actor_id and expiration in signed session
pub fn authenticate_actor(id: i64, req: &mut Request){
    deauth(req);

    req.session().set(Auth {
        actor_id: id,
        expiration_date: (unix_epoch()
                          + get_config::<u64>("SESSION_DURATION") * 60)
    }).unwrap();

    req.session().set(CsrfToken {
        token: generate_csrf_token()
    }).unwrap();
}

/// Clear signed session
pub fn deauth(req: &mut Request) {
    req.session().clear().unwrap();
}

/// Returns the hash of the password (libsodium pwhash)
pub fn hash_password(password: &str) -> String {
    // Normalize password
    let normalized_pw = basic_hash(password).into_bytes();

    pwhash::pwhash(&normalized_pw,
                   pwhash::OPSLIMIT_INTERACTIVE,
                   pwhash::MEMLIMIT_INTERACTIVE).unwrap()
                                                .as_ref()
                                                .to_hex()
}

/// Returns true if password matches the hash
pub fn check_password(password: &str, hash: &str) -> bool {
    // Normalize password
    let normalized_pw = basic_hash(password).into_bytes();

    // Turns Hex back into byte array
    let undigested_hash: Vec<u8> = FromHex::from_hex(hash.to_owned().into_bytes()).unwrap();
    let hash_obj = pwhash::HashedPassword::from_slice(&undigested_hash).unwrap();

    pwhash::pwhash_verify(&hash_obj, &normalized_pw)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Integration test to assure the cookies are being properly encoded/decoded
    fn session_storage() {
        let auth = Auth {
            actor_id: 1,
            expiration_date: unix_epoch()
        };
        let csrf_token  = CsrfToken {
            token: "3".to_owned()
        };

        {
            let raw = auth.clone().into_raw();
            let from_raw = Auth::from_raw(raw).unwrap();
            assert_eq!(auth, from_raw);
        }

        {
            let raw = csrf_token.clone().into_raw();
            let from_raw = CsrfToken::from_raw(raw).unwrap();
            assert_eq!(csrf_token, from_raw);
        }
    }

    #[test]
    /// Integration test to assure the password authentication is working
    fn password_handler() {
        assert!(check_password("hello", &hash_password("hello")));
        assert!(!check_password("hell", &hash_password("hello")));
    }
}
*/
