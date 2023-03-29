use crate::{utils, Device, User};
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Debug, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct AuthToken(pub String);

impl AuthToken {
    pub fn random() -> Self {
        Self(utils::random_string(64))
    }
}

impl AuthToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }
}

#[derive(sqlx::FromRow, Getters, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Auth {
    user: User,
    device: Option<Device>,
}
