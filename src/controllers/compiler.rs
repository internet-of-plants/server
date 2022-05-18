use crate::controllers::Result;
use crate::db::code_generation::{Compilation, CompilationId, Compiler, CompilerId};
use crate::db::sensor::SensorId;
use crate::db::target::TargetId;
use crate::extractor::Authorization;
use crate::prelude::*;
use axum::extract::{Extension, Json, Path};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewCompiler {
    pub target_id: TargetId,
    pub sensor_ids: Vec<SensorId>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompilerView {
    pub id: CompilerId,
    pub sensor_names: Vec<String>,
    pub target_arch: String,
}

impl CompilerView {
    async fn new(txn: &mut Transaction<'_>, compiler: Compiler) -> Result<Self> {
        let mut sensor_names = Vec::new();
        for sensor in compiler.sensors(txn).await? {
            sensor_names.push(sensor.prototype(txn).await?.name().to_owned());
        }
        Ok(Self {
            id: compiler.id(),
            sensor_names,
            target_arch: compiler.target(txn).await?.prototype(txn).await?.arch,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompilationView {
    pub id: CompilationId,
    pub compiler: CompilerView,
    pub platformio_ini: String,
    pub main_cpp: String,
    pub pin_hpp: String,
}

impl CompilationView {
    async fn new(txn: &mut Transaction<'_>, compilation: Compilation) -> Result<Self> {
        let compiler = compilation.compiler(txn).await?;
        Ok(Self {
            id: compilation.id(),
            platformio_ini: compilation.platformio_ini,
            main_cpp: compilation.main_cpp,
            pin_hpp: compilation.pin_hpp,
            compiler: CompilerView::new(txn, compiler).await?,
        })
    }
}

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
    Json(new_compiler): Json<NewCompiler>,
) -> Result<Json<CompilationView>> {
    let mut txn = pool.begin().await?;

    // TODO: filter by user
    let compiler = Compiler::new(&mut txn, new_compiler.target_id, new_compiler.sensor_ids).await?;
    let compilation = compiler.compile(&mut txn).await?;
    let view = CompilationView::new(&mut txn, compilation).await?;

    txn.commit().await?;
    Ok(Json(view))
}

pub async fn compilations(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<Vec<CompilationView>>> {
    let mut txn = pool.begin().await?;

    // TODO: filter by user
    let compilations = Compilation::list(&mut txn).await?;
    let mut views = Vec::new();
    for compilation in compilations {
        views.push(CompilationView::new(&mut txn, compilation).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}

pub async fn compile_firmware(
    Path(compilation_id): Path<CompilationId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<StatusCode> {
    let mut txn = pool.begin().await?;

    // TODO: filter by user
    let compilation = Compilation::find_by_id(&mut txn, compilation_id).await?;
    let _firmware = compilation.compile(&mut txn).await?;

    txn.commit().await?;
    Ok(StatusCode::OK)
}
