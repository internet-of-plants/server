use State;
use actix_web::{HttpRequest, middleware::session::RequestSession};
use hex::{FromHex, ToHex};
use lib::{error::Error, utils::basic_hash};
use sodiumoxide::crypto::pwhash;

/// Return user_id stored in session cookie
pub fn user_id(req: &HttpRequest<State>) -> Result<i32, Error> {
    match req.session().get::<i32>("user_id") {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Err(Error::NotAuthenticated),
        Err(_) => Err(Error::NotAuthenticated),
    }
}

/// Check authentication session cookie
pub fn is_auth(req: &HttpRequest<State>) -> Result<(), Error> {
    match req.session().get::<i32>("user_id") {
        Ok(Some(_)) => Ok(()),
        Ok(None) => Err(Error::NotAuthenticated),
        Err(_) => Err(Error::NotAuthenticated),
    }
}

/// Set session cookie
pub fn authenticate(req: &HttpRequest<State>, user_id: i32) -> Result<(), Error> {
    match req.session().set("user_id", user_id) {
        Ok(_) => Ok(()),
        Err(_) => Err(Error::NotAuthenticated),
    }
}

/// Returns the hash of the password (libsodium pwhash)
pub fn hash_password(password: &str) -> Result<String, Error> {
    let normalized_pw = basic_hash(password).into_bytes();

    Ok(pwhash::pwhash(
        &normalized_pw,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE,
    )?.as_ref()
        .to_hex())
}

/// Returns true if password matches the hash
pub fn check_password(password: &str, hash: &str) -> Result<bool, Error> {
    let normalized_pw = basic_hash(password).into_bytes();
    let undigested_hash: Vec<u8> = FromHex::from_hex(hash.to_owned().into_bytes())?;

    let hash_obj = match pwhash::HashedPassword::from_slice(&undigested_hash) {
        Some(v) => v,
        None => return Err(Error::SodiumOxide(())),
    };

    Ok(pwhash::pwhash_verify(&hash_obj, &normalized_pw))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password() {
        assert!(check_password("hello", &hash_password("hello").unwrap()).unwrap());
        assert!(!check_password("hell", &hash_password("hello").unwrap()).unwrap());
        assert!(!check_password(
            "hellooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo",
            &hash_password("helloooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo")
                .unwrap()).unwrap())
    }
}
