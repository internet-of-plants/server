use crate::{utils, Device, User};
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

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Auth {
    pub user: User,
    pub device: Option<Device>,
}
