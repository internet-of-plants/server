use hex::{FromHex, ToHex};
use sodiumoxide::crypto::pwhash;
use gotham::state::FromState;
use gotham::middleware::session::SessionData;
use lib::utils::{basic_hash, random_string};
use gotham::state::State;

#[derive(StateData, Serialize, Deserialize, Debug)]
pub struct Session {
    csrf_token: String,
    id: Option<i32>
}

#[derive(StateData, Deserialize, Debug)]
struct CsrfToken {
    csrf_token: String
}

pub fn is_auth(state: &State) -> bool {
    match **SessionData::<Option<Session>>::borrow_from(state) {
        Some(Session { csrf_token: _, id: Some(_) }) => true,
        _ => false
    }
}

pub fn is_csrf_valid(state: &mut State) -> bool {
    let form = __from_body!(state, CsrfToken);
    let cookie = csrf_token(&state);
    match (form, cookie) {
        (Some(CsrfToken { csrf_token: form }), Some(cookie)) => {
            *cookie == form
        },
        _ => false
    }
}

pub fn set_csrf_token(state: &mut State) {
    let session = SessionData::<Option<Session>>::borrow_mut_from(state);
    match **session {
        Some(_) => {},
        None => **session = Some(Session { csrf_token: random_string(30), id: None})
    }
}

pub fn csrf_token(state: &State) -> Option<&String> {
    match **SessionData::<Option<Session>>::borrow_from(state) {
        Some(Session { ref csrf_token, id: _}) => Some(csrf_token),
        None => None
    }
}

pub fn authenticate(state: &mut State, id: i32) {
    let session = SessionData::<Option<Session>>::borrow_mut_from(state);
    **session = Some(Session { csrf_token: random_string(30), id: Some(id)})
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

pub fn deauth(state: &mut State) {
    match SessionData::<Option<Session>>::take_from(state).discard(state) {
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Integration test to assure the password authentication is working
    fn password_handler() {
        assert!(check_password("hello", &hash_password("hello")));
        assert!(!check_password("hell", &hash_password("hello")));
    }
}

/*
use iron::Request;
use lib::utils::{unix_epoch, basic_hash, get_param, random_string};
use lib::serde_json;

/// Returns Result with actor_id stored in signed session (if auth, else None)
pub fn get_actor_id(req: &mut Request) -> Option<i64> {
    match req.session().get::<Auth>() {
        Ok(Some(auth)) => Some(auth.actor_id),
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
}
*/
