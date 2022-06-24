use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct TargetPrototypeId(pub i64);

impl TargetPrototypeId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TargetPrototype {
    pub id: TargetPrototypeId,
    pub arch: String,
    pub build_flags: String,
    pub build_unflags: Option<String>,
    pub platform: String,
    pub framework: Option<String>,
    pub platform_packages: Option<String>,
    pub extra_platformio_params: Option<String>,
    pub ldf_mode: Option<String>,
}

impl TargetPrototype {
    pub async fn new(
        txn: &mut Transaction<'_>,
        arch: String,
        build_flags: String,
        platform: String,
        framework: Option<String>,
        platform_packages: Option<String>,
        extra_platformio_params: Option<String>,
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
            build_flags,
            build_unflags: None,
            platform,
            framework,
            platform_packages,
            extra_platformio_params,
            ldf_mode,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: TargetPrototypeId) -> Result<Self> {
        Ok(sqlx::query_as(
            "SELECT id, arch, build_flags, build_unflags, platform, framework, platform_packages, extra_platformio_params, ldf_mode FROM target_prototypes WHERE id = $1"
        )
            .bind(&id)
            .fetch_one(&mut *txn)
            .await?)
    }

    pub async fn set_build_unflags(
        &mut self,
        txn: &mut Transaction<'_>,
        build_unflags: Option<String>,
    ) -> Result<()> {
        sqlx::query("UPDATE target_prototypes SET build_unflags = $1 WHERE id = $2")
            .bind(&build_unflags)
            .bind(&self.id)
            .execute(&mut *txn)
            .await?;
        self.build_unflags = build_unflags;
        Ok(())
    }

    pub fn id(&self) -> TargetPrototypeId {
        self.id
    }
}
