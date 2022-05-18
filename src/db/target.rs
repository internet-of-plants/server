use crate::db::board::*;
use crate::db::sensor::Dependency;
use crate::db::target_prototype::*;
use crate::db::user::UserId;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct TargetId(pub i64);

impl TargetId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub id: TargetId,
    owner_id: UserId,
    target_prototype_id: TargetPrototypeId,
    board_id: BoardId,
}

impl Target {
    pub fn id(&self) -> TargetId {
        self.id
    }

    pub async fn board(&self, txn: &mut Transaction<'_>) -> Result<Board> {
        Ok(Board::find_by_id(txn, self.board_id).await?)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, target_id: TargetId) -> Result<Target> {
        Ok(sqlx::query_as(
            "SELECT id, owner_id, target_prototype_id, board_id FROM targets WHERE id = $1",
        )
        .bind(&target_id)
        .fetch_one(&mut *txn)
        .await?)
    }

    pub async fn prototype(&self, txn: &mut Transaction<'_>) -> Result<TargetPrototype> {
        TargetPrototype::find_by_id(txn, self.target_prototype_id).await
    }

    pub async fn list_for_prototype(
        txn: &mut Transaction<'_>,
        owner_id: UserId,
        target_prototype_id: TargetPrototypeId,
    ) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, owner_id, target_prototype_id, board_id FROM targets WHERE target_prototype_id = $1 AND owner_id = $2",
        )
            .bind(&target_prototype_id)
            .bind(&owner_id)
            .fetch_all(&mut *txn)
        .await?)
    }

    pub async fn list(txn: &mut Transaction<'_>, owner_id: UserId) -> Result<Vec<Self>> {
        Ok(sqlx::query_as(
            "SELECT id, owner_id, target_prototype_id, board_id FROM targets WHERE owner_id = $1",
        )
        .bind(&owner_id)
        .fetch_all(&mut *txn)
        .await?)
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        board_id: BoardId,
        owner_id: UserId,
        target_prototype_id: TargetPrototypeId,
    ) -> Result<Self> {
        let (id,): (TargetId,) = sqlx::query_as(
            "INSERT INTO targets (board_id, target_prototype_id, owner_id) VALUES ($1, $2, $3) RETURNING id",
        )
        .bind(&board_id)
        .bind(&target_prototype_id)
        .bind(&owner_id)
        .fetch_one(&mut *txn)
        .await?;
        Ok(Self {
            id,
            owner_id,
            board_id,
            target_prototype_id,
        })
    }

    pub async fn compile_platformio_ini(
        &self,
        txn: &mut Transaction<'_>,
        lib_deps: &[Dependency],
    ) -> Result<String> {
        let prototype = self.prototype(&mut *txn).await?;
        let arch = &prototype.arch;
        let build_type = "debug".to_owned();
        //match prototype.kind {
        //    CompilationType::Debug => "debug",
        //    CompilationType::Release => "release",
        //};
        let framework = prototype
            .framework
            .as_ref()
            .map_or(String::new(), |f| format!("framework = {f}"));
        let platform = &prototype.platform;
        let board = self.board(&mut *txn).await?.board;
        let ldf_mode = prototype
            .ldf_mode
            .as_ref()
            .map_or(String::new(), |f| format!("lib_ldf_mode = {f}"));
        let build_flags = &prototype.build_flags;
        let extra_platformio_params = &prototype.extra_platformio_params;
        let platform_packages = &prototype.platform_packages;
        let lib_deps = lib_deps.join("\n    ");

        Ok(format!(
            "[env:{arch}-{board}]
build_type = {build_type}
build_flags =
    -O3
    -D IOP_LOG_LEVEL=iop::LogLevel::INFO

    -D ARDUINOJSON_ENABLE_ARDUINO_STRING=0
	-D ARDUINOJSON_ENABLE_ARDUINO_STREAM=0
	-D ARDUINOJSON_ENABLE_ARDUINO_PRINT=0
	-D ARDUINOJSON_ENABLE_PROGMEM=0

    {build_flags}
platform = {platform}
{framework}
board = {board}
{ldf_mode}
{extra_platformio_params}
lib_deps = 
    {lib_deps}
    https://github.com/internet-of-plants/iop
platform_packages = 
    {platform_packages}"
        ))
    }
}
