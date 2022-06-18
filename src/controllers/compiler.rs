use crate::controllers::Result;
use crate::db::code_generation::{Compilation, CompilationId, Compiler, CompilationView};
use crate::db::sensor::{NewSensor, Sensor};
use crate::db::target::TargetId;
use crate::extractor::Authorization;
use crate::{prelude::*, DeviceId, Device};
use axum::extract::{Extension, Json, Path};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewCompiler {
    pub device_id: DeviceId,
    pub target_id: TargetId,
    pub sensors: Vec<NewSensor>,
}

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
    Json(new_compiler): Json<NewCompiler>,
) -> Result<Json<CompilationView>> {
    let mut txn = pool.begin().await?;

    let mut sensor_ids = Vec::with_capacity(new_compiler.sensors.len());
    for sensor in new_compiler.sensors {
        sensor_ids.push(Sensor::new(&mut txn, sensor).await?.id);
    }
    let (compiler, compilation) = Compiler::new(&mut txn, new_compiler.target_id, sensor_ids).await?;
    // TODO: check ownership
    let mut device = Device::find_by_id(&mut txn, &new_compiler.device_id).await?;
    device.set_compiler_id(&mut txn, Some(compiler.id())).await?;
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
