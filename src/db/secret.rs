use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SecretAlgo {
    LibsodiumSealedBox,
}
