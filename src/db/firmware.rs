use crate::db::compilation::{Compilation, CompilationId};
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareView {
    id: FirmwareId,
    hash: String,
}

impl FirmwareView {
    pub fn new(firmware: Firmware) -> Self {
        Self {
            id: firmware.id(),
            hash: firmware.hash().to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct FirmwareId(i64);

impl FirmwareId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Firmware {
    id: FirmwareId,
    compilation_id: Option<CompilationId>,
    binary_hash: String,
}

impl Firmware {
    pub async fn new_unknown(txn: &mut Transaction<'_>, binary_hash: String) -> Result<Self> {
        let (id,): (FirmwareId,) =
            sqlx::query_as("INSERT INTO firmwares (binary_hash) VALUES ($1) RETURNING id")
                .bind(&binary_hash)
                .fetch_one(txn)
                .await?;
        Ok(Self {
            id,
            compilation_id: None,
            binary_hash,
        })
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        compilation: &Compilation,
        bin: Vec<u8>,
    ) -> Result<Self> {
        // TODO: move to SHA-256
        let md5 = md5::compute(&bin);
        let md5 = &*md5;
        let mut binary_hash = String::with_capacity(md5.len() * 2);
        for byte in md5 {
            write!(binary_hash, "{:02X}", byte)?;
        }

        let id: Option<(FirmwareId,)> = sqlx::query_as(
            "
            SELECT id
            FROM firmwares
            WHERE compilation_id = $1 AND binary_hash = $2",
        )
        .bind(compilation.id())
        .bind(&binary_hash)
        .fetch_optional(&mut *txn)
        .await?;

        let id = if let Some((id,)) = id {
            id
        } else {
            let (id,): (FirmwareId,) = sqlx::query_as(
                "INSERT INTO firmwares (compilation_id, bin, binary_hash) VALUES ($1, $2, $3) RETURNING id",
            )
            .bind(compilation.id())
            .bind(&bin)
            .bind(&binary_hash)
            .fetch_one(txn)
            .await?;
            id
        };
        Ok(Self {
            id,
            compilation_id: Some(compilation.id()),
            binary_hash,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: FirmwareId) -> Result<Self> {
        let firmware = sqlx::query_as(
            "SELECT id, compilation_id, binary_hash FROM firmwares WHERE id = $1",
        )
        .bind(id)
        .fetch_one(txn)
        .await?;
        Ok(firmware)
    }

    pub async fn try_find_by_hash(txn: &mut Transaction<'_>, hash: &str) -> Result<Option<Self>> {
        let firmware = sqlx::query_as(
            "SELECT id, compilation_id, binary_hash FROM firmwares WHERE binary_hash = $1",
        )
        .bind(hash)
        .fetch_optional(txn)
        .await?;
        Ok(firmware)
    }

    pub async fn find_by_compilation(
        txn: &mut Transaction<'_>,
        compilation: &Compilation,
    ) -> Result<Self> {
        let firmware = sqlx::query_as(
            "SELECT id, compilation_id, binary_hash FROM firmwares WHERE compilation_id = $1",
        )
        .bind(compilation.id())
        .fetch_one(txn)
        .await?;
        Ok(firmware)
    }

    pub fn id(&self) -> FirmwareId {
        self.id
    }

    pub fn hash(&self) -> &str {
        &self.binary_hash
    }

    pub async fn bin(&self, txn: &mut Transaction<'_>) -> Result<Option<Vec<u8>>> {
        let bin = sqlx::query_as("SELECT bin FROM firmwares WHERE id = $1")
            .bind(self.id)
            .fetch_optional(txn)
            .await?;
        Ok(bin.map(|(bin,)| bin))
    }

    pub async fn compilation(&self, txn: &mut Transaction<'_>) -> Result<Option<Compilation>> {
        match self.compilation_id {
            Some(id) => Ok(Some(Compilation::find_by_id(txn, id).await?)),
            None => Ok(None),
        }
    }
}
