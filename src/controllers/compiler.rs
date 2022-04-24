use crate::controllers::Result;
use crate::db::code_generation::{Compiled, CompiledId, Compiler, CompilerId};
use crate::db::sensor::SensorId;
use crate::db::target::TargetId;
use crate::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewCompiler {
    target_id: TargetId,
    sensor_ids: Vec<SensorId>,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CompilerView {
    id: CompilerId,
    sensor_names: Vec<String>,
    target_arch: String,
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CompiledView {
    id: CompiledId,
    compiler: CompilerView,
    platformio_ini: String,
    main_cpp: String,
    pin_hpp: String,
}

impl CompiledView {
    async fn new(txn: &mut Transaction<'_>, compiled: Compiled) -> Result<Self> {
        let compiler = compiled.compiler(txn).await?;
        Ok(Self {
            id: compiled.id(),
            platformio_ini: compiled.platformio_ini,
            main_cpp: compiled.main_cpp,
            pin_hpp: compiled.pin_hpp,
            compiler: CompilerView::new(txn, compiler).await?,
        })
    }
}

pub async fn new(
    pool: &'static Pool,
    _auth: Auth,
    new_compiler: NewCompiler,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    // TODO: filter by user
    let compiler = Compiler::new(&mut txn, new_compiler.target_id, new_compiler.sensor_ids).await?;
    let compiled = compiler.compile(&mut txn).await?;
    let view = CompiledView::new(&mut txn, compiled).await?;

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&view))
}

pub async fn compilations(
    pool: &'static Pool,
    _auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    // TODO: filter by user
    let compileds = Compiled::list(&mut txn).await?;
    let mut views = Vec::new();
    for compiled in compileds {
        views.push(CompiledView::new(&mut txn, compiled).await?);
    }

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&views))
}

pub async fn compile_firmware(
    compiled_id: CompiledId,
    pool: &'static Pool,
    _auth: Auth,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    // TODO: filter by user
    let compiled = Compiled::find_by_id(&mut txn, compiled_id).await?;
    let _firmware = compiled.compile(&mut txn).await?;

    txn.commit().await.map_err(Error::from)?;
    Ok("")
}
