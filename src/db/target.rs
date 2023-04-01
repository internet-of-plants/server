use crate::{
    Dependency, DeviceConfigRequest, DeviceConfigRequestView, NewDeviceConfigRequest, Result,
    TargetPrototype, TargetPrototypeId, Transaction,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetView {
    #[copy]
    id: TargetId,
    arch: String,
    name: Option<String>,
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
            name: target.name().to_owned(),
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
    pub name: Option<String>,
    board: Option<String>,
    #[copy]
    target_prototype_id: TargetPrototypeId,
    pin_hpp: String,
    build_flags: Option<String>,
}

impl Target {
    pub async fn new(
        txn: &mut Transaction<'_>,
        board: Option<String>,
        pins: Vec<String>,
        pin_hpp: String,
        target_prototype: &TargetPrototype,
        new_config_requests: Vec<NewDeviceConfigRequest>,
    ) -> Result<Self> {
        let (id,): (TargetId,) = sqlx::query_as(
            "INSERT INTO targets (board, target_prototype_id, pin_hpp) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&board)
        .bind(target_prototype.id())
        .bind(&pin_hpp)
        .fetch_one(&mut *txn)
        .await?;
        for pin in pins {
            sqlx::query("INSERT INTO pins (target_id, name) VALUES ($1, $2)")
                .bind(id)
                .bind(pin)
                .execute(&mut *txn)
                .await?;
        }
        let target = Self {
            id,
            name: None,
            board,
            pin_hpp,
            target_prototype_id: target_prototype.id(),
            build_flags: None,
        };
        for config_request in new_config_requests {
            DeviceConfigRequest::new(
                txn,
                config_request.name,
                config_request.human_name,
                config_request.type_name,
                config_request.widget,
                &target,
                config_request.secret_algo,
            )
            .await?;
        }
        Ok(target)
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, name, board, target_prototype_id, pin_hpp, build_flags
            FROM targets",
        )
        .fetch_all(txn)
        .await?)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: TargetId) -> Result<Self> {
        let target = sqlx::query_as(
            "SELECT id, name, board, target_prototype_id, pin_hpp, build_flags FROM targets WHERE id = $1",
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

    pub async fn set_name(
        &mut self,
        txn: &mut Transaction<'_>,
        name: Option<String>,
    ) -> Result<()> {
        sqlx::query("UPDATE targets SET name = $1 WHERE id = $2")
            .bind(&name)
            .bind(self.id)
            .execute(txn)
            .await?;
        self.name = name;
        Ok(())
    }

    pub async fn set_build_flags(
        &mut self,
        txn: &mut Transaction<'_>,
        build_flags: Option<String>,
    ) -> Result<()> {
        sqlx::query("UPDATE targets SET build_flags = $1 WHERE id = $2")
            .bind(&build_flags)
            .bind(self.id)
            .execute(txn)
            .await?;
        self.build_flags = build_flags;
        Ok(())
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
        lib_deps: &[Dependency],
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
        let mut lib_deps = lib_deps.to_owned();
        lib_deps.sort_unstable_by_key(|d| d.repo_url().clone());
        let lib_deps = lib_deps
            .into_iter()
            .map(|d| format!("{}#{}", d.repo_url(), d.branch()))
            .collect::<Vec<String>>()
            .join("\n    ");
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
lib_deps =
    {lib_deps}
    https://github.com/internet-of-plants/iop\
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
