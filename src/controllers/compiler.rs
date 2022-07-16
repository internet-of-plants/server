use crate::{
    extractor::User, CompilationView, Compiler, Device, DeviceConfig, NewCompiler, Pool, Result,
    Sensor, Target,
};
use axum::extract::{Extension, Json};

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(new_compiler): Json<NewCompiler>,
) -> Result<Json<CompilationView>> {
    let mut txn = pool.begin().await?;

    let mut device = Device::find_by_id(&mut txn, new_compiler.device_id, &user).await?;
    let collection = device.collection(&mut txn).await?;
    let organization = collection.organization(&mut txn).await?;

    let mut sensors_and_alias = Vec::with_capacity(new_compiler.sensors.len());
    for sensor in new_compiler.sensors {
        let alias = sensor.alias.clone();
        let sensor = Sensor::new(&mut txn, sensor).await?;
        sensors_and_alias.push((sensor, alias));
    }
    let mut device_configs = Vec::new();
    for config in new_compiler.device_configs {
        let config = DeviceConfig::new(&mut txn, config, &organization).await?;
        device_configs.push(config);
    }
    // TODO: enforce required configs
    // TODO: encrypt secrets
    let target = Target::find_by_id(&mut txn, new_compiler.target_id).await?;
    let (compiler, compilation) = Compiler::new(
        &mut txn,
        &target,
        sensors_and_alias,
        device_configs,
        &organization,
    )
    .await?;

    device.set_compiler(&mut txn, Some(&compiler)).await?;

    let view = CompilationView::new(&mut txn, compilation).await?;

    txn.commit().await?;
    Ok(Json(view))
}
