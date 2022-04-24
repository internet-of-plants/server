use crate::db::code_generation::{Compiled, CompiledId};
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct FirmwareId(i64);

impl FirmwareId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow)]
pub struct Firmware {
    id: FirmwareId,
    compiled_id: Option<CompiledId>,
    binary: Vec<u8>,
    binary_hash: String,
}

impl Firmware {
    pub fn id(&self) -> FirmwareId {
        self.id
    }

    pub async fn compiled(&self, txn: &mut Transaction<'_>) -> Result<Option<Compiled>> {
        match self.compiled_id {
            Some(id) => Ok(Some(Compiled::find_by_id(txn, id).await?)),
            None => Ok(None),
        }
    }

    pub fn hash(&self) -> &str {
        &self.binary_hash
    }

    pub fn into_binary(self) -> Vec<u8> {
        self.binary
    }

    pub fn binary(&self) -> &[u8] {
        &self.binary
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        compiled_id: Option<CompiledId>,
        binary: Vec<u8>,
    ) -> Result<Self> {
        // TODO: move to SHA-256
        let md5 = md5::compute(&binary);
        let md5 = &*md5;
        let mut binary_hash = String::with_capacity(md5.len() * 2);
        for byte in md5 {
            write!(binary_hash, "{:02X}", byte).map_err(Error::from)?;
        }

        let (id,): (FirmwareId,) = sqlx::query_as(
            "INSERT INTO compilations (compiled_id, bin, binary_hash) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(compiled_id)
        .bind(&binary)
        .bind(&binary_hash)
        .fetch_one(&mut *txn)
        .await?;
        Ok(Self {
            id,
            compiled_id,
            binary,
            binary_hash,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: FirmwareId) -> Result<Self> {
        let firmware = sqlx::query_as(
            " SELECT id, compiled_id, firmware, firmware_hash FROM firmwares WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(firmware)
    }
}
