use crate::{Compilation, CompilationId, Device, Organization, Result, Transaction};
use derive_get::Getters;
use derive::id;
use serde::{Deserialize, Serialize};
use std::fmt::Write;

#[derive(Getters, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FirmwareView {
    #[copy]
    id: FirmwareId,
    hash: String,
}

impl FirmwareView {
    pub fn new(firmware: Firmware) -> Self {
        Self {
            id: firmware.id(),
            hash: firmware.binary_hash().to_owned(),
        }
    }
}

#[id]
pub struct FirmwareId;

#[derive(sqlx::FromRow, Getters, Debug)]
pub struct Firmware {
    #[copy]
    id: FirmwareId,
    #[copy]
    compilation_id: Option<CompilationId>,
    binary_hash: String,
}

impl Firmware {
    pub async fn new_unknown(
        txn: &mut Transaction<'_>,
        binary_hash: String,
        organization: &Organization,
    ) -> Result<Self> {
        let (id,): (FirmwareId,) = sqlx::query_as(
            "INSERT INTO firmwares (binary_hash, organization_id) VALUES ($1, $2) RETURNING id",
        )
        .bind(&binary_hash)
        .bind(organization.id())
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
        let compiler = compilation.compiler(txn).await?;
        let organization = compiler.organization(txn).await?;

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
            WHERE organization_id = $1 AND binary_hash = $2",
        )
        .bind(organization.id())
        .bind(&binary_hash)
        .fetch_optional(&mut *txn)
        .await?;

        let id = if let Some((id,)) = id {
            id
        } else {
            let (id,): (FirmwareId,) = sqlx::query_as(
                "INSERT INTO firmwares (compilation_id, organization_id, bin, binary_hash) VALUES ($1, $2, $3, $4) RETURNING id",
            )
            .bind(compilation.id())
            .bind(organization.id())
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

    pub async fn find_by_device(txn: &mut Transaction<'_>, device: &Device) -> Result<Self> {
        let firmware = sqlx::query_as(
            "SELECT firmwares.id, compilation_id, binary_hash
                 FROM firmwares
                 WHERE firmwares.id = $1",
        )
        .bind(device.firmware_id())
        .fetch_one(txn)
        .await?;
        Ok(firmware)
    }

    pub async fn try_find_by_hash(
        txn: &mut Transaction<'_>,
        organization: &Organization,
        hash: &str,
    ) -> Result<Option<Self>> {
        let firmware = sqlx::query_as(
            "SELECT firmwares.id, compilation_id, binary_hash
             FROM firmwares
             INNER JOIN devices ON devices.firmware_id = firmwares.id
             INNER JOIN collections ON collections.id = devices.collection_id
             INNER JOIN collection_belongs_to_organization cbt ON cbt.collection_id = collections.id
             WHERE binary_hash = $1 AND cbt.organization_id = $2
             UNION
             SELECT firmwares.id, compilation_id, binary_hash
             FROM firmwares
             INNER JOIN compilations ON compilations.id = firmwares.compilation_id
             INNER JOIN compilers ON compilers.id = compilations.compiler_id
             WHERE binary_hash = $1 AND compilers.organization_id = $2
",
        )
        .bind(hash)
        .bind(organization.id())
        .fetch_optional(txn)
        .await?;
        Ok(firmware)
    }

    pub async fn latest_by_compilation(
        txn: &mut Transaction<'_>,
        compilation: &Compilation,
    ) -> Result<Self> {
        let firmware = sqlx::query_as(
            "SELECT id, compilation_id, binary_hash FROM firmwares WHERE compilation_id = $1 ORDER BY created_at DESC",
        )
        .bind(compilation.id())
        .fetch_one(txn)
        .await?;
        Ok(firmware)
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
            Some(id) => Ok(Some(Compilation::find_by_id(txn, self, id).await?)),
            None => Ok(None),
        }
    }
}
