use crate::{
    Dependency, DeviceConfigRequest, DeviceConfigRequestView, NewDeviceConfigRequest, Result,
    TargetPrototype, TargetPrototypeId, Transaction,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Getters, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetView {
    #[copy]
    id: TargetId,
    arch: String,
    build_flags: String,
    platform: String,
    framework: Option<String>,
    platform_packages: Option<String>,
    extra_platformio_params: Option<String>,
    ldf_mode: Option<String>,
    board: Option<String>,
    configuration_requests: Vec<DeviceConfigRequestView>,
}

impl TargetView {
    pub async fn new(txn: &mut Transaction<'_>, target: Target) -> Result<Self> {
        let prototype = target.prototype(txn).await?;
        let mut configuration_requests = Vec::new();
        for config_request in target.configuration_requests(txn).await? {
            configuration_requests.push(DeviceConfigRequestView::new(txn, config_request).await?);
        }
        Ok(Self {
            id: target.id(),
            arch: prototype.arch().clone(),
            build_flags: prototype.build_flags().clone(),
            platform: prototype.platform().clone(),
            framework: prototype.framework().clone(),
            platform_packages: prototype.platform_packages().clone(),
            extra_platformio_params: prototype.extra_platformio_params().clone(),
            ldf_mode: prototype.ldf_mode().clone(),
            board: target.board().clone(),
            configuration_requests,
        })
    }
}

#[id]
pub struct TargetId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Target {
    #[copy]
    id: TargetId,
    board: Option<String>,
    #[copy]
    target_prototype_id: TargetPrototypeId,
    pin_hpp: String,
    build_flags: Option<String>,
}

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NewTarget {
    #[serde(default)]
    board: Option<String>,
    pins: Vec<String>,
    pin_hpp: String,
    target_prototype_arch: String,
    config_requests: Vec<NewDeviceConfigRequest>,
    #[serde(default)]
    build_flags: Option<String>,
}

impl TryFrom<serde_json::Value> for NewTarget {
    type Error = serde_json::Error;

    fn try_from(json: serde_json::Value) -> Result<Self, Self::Error> {
        serde_json::from_value(json)
    }
}

impl Target {
    pub async fn new(txn: &mut Transaction<'_>, target: NewTarget) -> Result<Self> {
        let target_prototype =
            TargetPrototype::find_by_arch(txn, &target.target_prototype_arch).await?;

        let maybe: Option<(TargetId,)> =
            sqlx::query_as("SELECT id FROM targets WHERE target_prototype_id = $1")
                .bind(target_prototype.id())
                .fetch_optional(&mut *txn)
                .await?;

        let id = if let Some((id,)) = maybe {
            sqlx::query(
                "UPDATE targets SET pin_hpp = $2, build_flags = $3, board = $4 WHERE target_prototype_id = $1",
            )
            .bind(target_prototype.id())
            .bind(target.pin_hpp())
            .bind(target.build_flags())
            .bind(target.board())
            .execute(&mut *txn)
            .await?;
            id
        } else {
            let (id,): (TargetId,) = sqlx::query_as(
                "INSERT INTO targets (target_prototype_id, pin_hpp, build_flags, board) VALUES ($1, $2, $3, $4) RETURNING id"
            )
            .bind(target_prototype.id())
            .bind(target.pin_hpp())
            .bind(target.build_flags())
            .bind(target.board())
            .fetch_one(&mut *txn)
            .await?;
            id
        };

        let config_requests = target.config_requests;
        let pins = target.pins;

        let target = Self {
            id,
            board: target.board,
            pin_hpp: target.pin_hpp,
            target_prototype_id: target_prototype.id(),
            build_flags: target.build_flags,
        };

        let mut existing_pins = target.pins(txn).await?;

        for pin in pins {
            existing_pins.retain(|p| p != &pin);

            sqlx::query(
                "INSERT INTO pins (target_id, name) VALUES ($1, $2) ON CONFLICT (target_id, name) DO NOTHING",
            )
            .bind(id)
            .bind(pin)
            .execute(&mut *txn)
            .await?;
        }

        // TODO: deleting pins should invalidate compilers that use said pin
        if !existing_pins.is_empty() {
            panic!("Deleting pins is not supported");
        }
        // for pin in existing_pins {
        //    sqlx::query("DELETE FROM pins WHERE target_id = $1 AND name = $2")
        //        .bind(id)
        //        .bind(pin)
        //        .execute(&mut *txn)
        //        .await?;
        //}

        for config_request in config_requests {
            DeviceConfigRequest::new(txn, &config_request, &target).await?;
        }
        Ok(target)
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, board, target_prototype_id, pin_hpp, build_flags FROM targets",
        )
        .fetch_all(txn)
        .await?)
    }

    pub async fn list_for_prototype(
        txn: &mut Transaction<'_>,
        prototype_id: TargetPrototypeId,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, board, target_prototype_id, pin_hpp, build_flags
            FROM targets WHERE target_prototype_id = $1",
        )
        .bind(prototype_id)
        .fetch_all(txn)
        .await?)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: TargetId) -> Result<Self> {
        let target = sqlx::query_as(
            "SELECT id, board, target_prototype_id, pin_hpp, build_flags FROM targets WHERE id = $1",
        )
        .bind(id)
        .fetch_one(txn)
        .await?;
        Ok(target)
    }

    pub async fn pins(&self, txn: &mut Transaction<'_>) -> Result<Vec<String>> {
        let pins = sqlx::query_as("SELECT name FROM pins WHERE target_id = $1")
            .bind(self.id)
            .fetch_all(txn)
            .await?
            .into_iter()
            .map(|(name,)| name)
            .collect();
        Ok(pins)
    }

    pub async fn prototype(&self, txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
        TargetPrototype::find_by_id(txn, self.target_prototype_id).await
    }

    pub async fn configuration_requests(
        &self,
        txn: &mut Transaction<'_>,
    ) -> Result<Vec<DeviceConfigRequest>> {
        DeviceConfigRequest::find_by_target(txn, self).await
    }

    pub async fn compile_platformio_ini(
        &self,
        txn: &mut Transaction<'_>,
        mut lib_deps: Vec<Dependency>,
    ) -> Result<String> {
        let prototype = self.prototype(txn).await?;
        let arch = prototype.arch();
        let build_type = "release".to_owned();
        let framework = prototype
            .framework()
            .as_ref()
            .map_or(String::new(), |f| format!("framework = {f}\n"));
        let platform = prototype.platform();
        let board = self.board();
        let ldf_mode = prototype
            .ldf_mode()
            .as_ref()
            .map_or(String::new(), |f| format!("lib_ldf_mode = {f}\n"));
        let mut build_flags = prototype.build_flags().clone();
        if let Some(flags) = self.build_flags() {
            build_flags.push_str("\n    ");
            build_flags.push_str(flags);
        }
        let extra_platformio_params = prototype.extra_platformio_params();
        let platform_packages = prototype.platform_packages();

        lib_deps.dedup_by_key(|d| d.repo_url().clone());
        lib_deps.sort_unstable_by_key(|d| d.repo_url().clone());
        let mut lib_deps = lib_deps
            .into_iter()
            .map(|d| format!("{}#{}", d.repo_url(), d.branch()))
            .collect::<Vec<String>>()
            .join("\n    ");
        if !lib_deps.is_empty() {
            lib_deps.insert_str(0, "\n    ");
        }
        let mut env_name = vec![arch.as_str()];
        let board = if let Some(board) = board {
            env_name.push(board.as_str());
            format!("board = {board}\n")
        } else {
            String::new()
        };
        let env_name = env_name.join("-");

        let debug = if cfg!(debug_assertions) {
            "    -D IOP_DEBUG\n"
        } else {
            ""
        };

        Ok(format!(
            "[env:{env_name}]
build_flags =
    -D ARDUINOJSON_ENABLE_ARDUINO_STRING=0
    -D ARDUINOJSON_ENABLE_ARDUINO_STREAM=0
    -D ARDUINOJSON_ENABLE_ARDUINO_PRINT=0
    -D ARDUINOJSON_ENABLE_PROGMEM=0

    -std=c++17
    -O3
    -Wall
    {build_flags}
    -D IOP_LOG_LEVEL=iop::LogLevel::INFO
{debug}\
platform = {platform}
build_type = {build_type}
{framework}\
{board}\
{ldf_mode}\
{}\
lib_deps ={lib_deps}
    https://github.com/internet-of-plants/iop#main\
{}",
            extra_platformio_params
                .as_ref()
                .map_or_else(String::new, |p| format!("{p}\n")),
            platform_packages
                .as_ref()
                .map(|p| format!("\nplatform_packages =\n    {p}"))
                .unwrap_or_else(String::new)
        ))
    }
}
