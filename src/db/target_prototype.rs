use crate::db::board::Board;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct TargetPrototypeId(i64);

impl TargetPrototypeId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TargetPrototype {
    pub id: TargetPrototypeId,
    pub arch: String,
    //kind: CompilationType,
    pub build_flags: String,
    pub platform: String,
    pub framework: Option<String>,
    pub platform_packages: String,
    pub extra_platformio_params: String,
    pub ldf_mode: Option<String>,
}

impl TargetPrototype {
    pub async fn new(
        txn: &mut Transaction<'_>,
        arch: String,
        //kind: CompilationType,
        build_flags: String,
        platform: String,
        framework: Option<String>,
        platform_packages: String,
        extra_platformio_params: String,
        ldf_mode: Option<String>,
    ) -> Result<Self> {
        let (id,): (TargetPrototypeId,) = sqlx::query_as(
            "INSERT INTO target_prototypes (arch, build_flags, platform, framework, platform_packages, extra_platformio_params, ldf_mode) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        )
            .bind(&arch)
            .bind(&build_flags)
            .bind(&platform)
            .bind(&framework)
            .bind(&platform_packages)
            .bind(&extra_platformio_params)
            .bind(&ldf_mode)
            .fetch_one(&mut *txn)
            .await?;
        Ok(Self {
            id,
            arch,
            //kind,
            build_flags,
            platform,
            framework,
            platform_packages,
            extra_platformio_params,
            ldf_mode,
        })
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, arch, build_flags, platform, framework, platform_packages, extra_platformio_params, ldf_mode FROM target_prototypes"
        )
            .fetch_all(&mut *txn)
            .await?)
    }

    pub async fn boards(&self, txn: &mut Transaction<'_>) -> Result<Vec<Board>> {
        Board::list_by_target_prototype(txn, self.id).await
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: TargetPrototypeId) -> Result<Self> {
        Ok(sqlx::query_as(
            "SELECT id, arch, build_flags, platform, framework, platform_packages, extra_platformio_params, ldf_mode FROM target_prototypes WHERE id = $1"
        )
            .bind(&id)
            .fetch_one(&mut *txn)
            .await?)
    }
}
