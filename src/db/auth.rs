use crate::{utils, Device, User};
use derive_get::Getters;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use axum::response::IntoResponse;

#[derive(Serialize, Deserialize, Display, sqlx::Type, Clone, Debug, PartialEq, Eq)]
#[sqlx(transparent)]
pub struct AuthToken(String);

impl IntoResponse for AuthToken {
    fn into_response(self) -> axum::response::Response {
        self.0.into_response()
    }
}

impl From<String> for AuthToken {
    fn from(s: String) -> Self {
        Self(s)
    }
}

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
