use crate::{Result, Transaction};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt::Write};

#[id]
pub struct CertificateId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Certificate {
    #[copy]
    id: CertificateId,
    #[copy]
    target_prototype_id: TargetPrototypeId,
    hash: String,
}

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewDependency {
    repo_url: String,
    branch: String,
}

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    repo_url: String,
    branch: String,
}

impl Dependency {
    pub fn new(repo_url: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            repo_url: repo_url.into(),
            branch: branch.into(),
        }
    }
}

#[id]
pub struct TargetPrototypeId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TargetPrototype {
    #[copy]
    id: TargetPrototypeId,
    certs_url: String,
    arch: String,
    build_flags: String,
    build_unflags: Option<String>,
    platform: String,
    framework: Option<String>,
    platform_packages: Option<String>,
    extra_platformio_params: Option<String>,
    ldf_mode: Option<String>,
}

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewTargetPrototype {
    certs_url: String,
    arch: String,
    build_flags: String,
    #[serde(default)]
    build_unflags: Option<String>,
    platform: String,
    #[serde(default)]
    framework: Option<String>,
    #[serde(default)]
    platform_packages: Option<String>,
    #[serde(default)]
    extra_platformio_params: Option<String>,
    #[serde(default)]
    ldf_mode: Option<String>,
    #[serde(default)]
    dependencies: Vec<NewDependency>,
}

impl TryFrom<serde_json::Value> for NewTargetPrototype {
    type Error = serde_json::Error;

    fn try_from(json: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(json)
    }
}

impl TargetPrototype {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(txn: &mut Transaction<'_>, prototype: NewTargetPrototype) -> Result<Self> {
        sqlx::query(
            "INSERT INTO target_prototypes
            (certs_url, arch, build_flags, platform, framework, platform_packages, extra_platformio_params, ldf_mode) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (arch)
            DO UPDATE SET certs_url = $1,
                          build_flags = $3,
                          platform = $4,
                          framework = $5,
                          platform_packages = $6,
                          extra_platformio_params = $7,
                          ldf_mode = $8",
        )
            .bind(prototype.certs_url())
            .bind(prototype.arch())
            .bind(prototype.build_flags())
            .bind(prototype.platform())
            .bind(prototype.framework())
            .bind(prototype.platform_packages())
            .bind(prototype.extra_platformio_params())
            .bind(prototype.ldf_mode())
            .execute(&mut *txn)
            .await?;

        let result = Self::find_by_arch(txn, prototype.arch()).await?;
        let mut dependencies = result.dependencies(txn).await?;

        for dependency in prototype.dependencies() {
            dependencies.retain(|d| d.repo_url() != dependency.repo_url());

            sqlx::query(
                "INSERT INTO dependency_belongs_to_target_prototype
                (target_prototype_id, repo_url, branch)
                VALUES ($1, $2, $3)
                ON CONFLICT (target_prototype_id, repo_url)
                DO UPDATE SET branch = $3",
            )
            .bind(result.id())
            .bind(dependency.repo_url())
            .bind(dependency.branch())
            .execute(&mut *txn)
            .await?;
        }

        for dependency in dependencies {
            sqlx::query(
                "DELETE FROM dependency_belongs_to_target_prototype
                WHERE target_prototype_id = $1 AND repo_url = $2",
            )
            .bind(result.id())
            .bind(dependency.repo_url())
            .execute(&mut *txn)
            .await?;
        }

        Ok(result)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: TargetPrototypeId) -> Result<Self> {
        Ok(sqlx::query_as(
            "SELECT id, certs_url, arch, build_flags, build_unflags, platform, framework, platform_packages, extra_platformio_params, ldf_mode FROM target_prototypes WHERE id = $1"
        )
            .bind(id)
            .fetch_one(txn)
            .await?)
    }

    pub async fn try_find_by_arch(txn: &mut Transaction<'_>, arch: &str) -> Result<Option<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, certs_url, arch, build_flags, build_unflags, platform, framework, platform_packages, extra_platformio_params, ldf_mode FROM target_prototypes WHERE arch = $1"
        )
            .bind(arch)
            .fetch_optional(txn)
            .await?)
    }

    pub async fn find_by_arch(txn: &mut Transaction<'_>, arch: &str) -> Result<Self> {
        Ok(sqlx::query_as(
            "SELECT id, certs_url, arch, build_flags, build_unflags, platform, framework, platform_packages, extra_platformio_params, ldf_mode FROM target_prototypes WHERE arch = $1"
        )
            .bind(arch)
            .fetch_one(txn)
            .await?)
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, certs_url, arch, build_flags, build_unflags, platform, framework, platform_packages, extra_platformio_params, ldf_mode FROM target_prototypes"
        )
            .fetch_all(txn)
            .await?)
    }

    pub async fn dependencies(&self, txn: &mut Transaction<'_>) -> Result<Vec<Dependency>> {
        let mut deps = vec![
            Dependency {
                repo_url: "https://github.com/bblanchon/ArduinoJson".to_owned(),
                branch: "6.x".to_owned(),
            },
            Dependency {
                repo_url: "https://github.com/internet-of-plants/iop-hal".to_owned(),
                branch: "main".to_owned(),
            },
            Dependency {
                repo_url: "https://github.com/internet-of-plants/iop".to_owned(),
                branch: "main".to_owned(),
            },
        ];

        let dependencies: Vec<Dependency> = sqlx::query_as(
            "SELECT repo_url, branch FROM dependency_belongs_to_target_prototype WHERE target_prototype_id = $1",
        )
        .bind(self.id)
        .fetch_all(txn)
        .await?;

        deps.extend(dependencies);

        Ok(deps)
    }

    pub async fn update_certificates(&self, txn: &mut Transaction<'_>) -> Result<()> {
        let content = reqwest::get(&self.certs_url).await?.text().await?;
        let md5 = md5::compute(&content);
        let md5 = &*md5;
        let mut hash = String::with_capacity(md5.len() * 2);
        for byte in md5 {
            write!(hash, "{:02X}", byte)?;
        }
        sqlx::query(
            "INSERT INTO certificates (target_prototype_id, hash, payload) VALUES ($1, $2, $3)
             ON CONFLICT (target_prototype_id, hash) DO NOTHING",
        )
        .bind(self.id)
        .bind(&hash)
        .bind(content.as_bytes())
        .execute(txn)
        .await?;
        Ok(())
    }

    pub async fn latest_certificates(txn: &mut Transaction<'_>) -> Result<Vec<Certificate>> {
        let certificates = sqlx::query_as(
            "SELECT DISTINCT ON (target_prototype_id) target_prototype_id, id, hash
            FROM certificates
            ORDER BY target_prototype_id, created_at DESC",
        )
        .fetch_all(txn)
        .await?;
        Ok(certificates)
    }

    pub async fn latest_certificate(&self, txn: &mut Transaction<'_>) -> Result<Certificate> {
        let certificate: Certificate = sqlx::query_as(
            "SELECT id, hash, target_prototype_id
            FROM certificates WHERE target_prototype_id = $1
            ORDER BY created_at DESC",
        )
        .bind(self.id)
        .fetch_one(txn)
        .await?;
        Ok(certificate)
    }
}
