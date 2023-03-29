use serde::{Deserialize, Serialize};

#[derive(sqlx::Type, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Copy)]
pub enum SecretAlgo {
    LibsodiumSealedBox,
}
