use crate::controllers::Result;
use crate::db::compilation::CompilationView;
use crate::db::compiler::Compiler;
use crate::db::sensor::{NewSensor, Sensor};
use crate::db::target::{Target, TargetId};
use crate::extractor::User;
use crate::{prelude::*, Device, DeviceId};
use axum::extract::{Extension, Json};
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
    User(user): User,
    Json(new_compiler): Json<NewCompiler>,
) -> Result<Json<CompilationView>> {
    let mut txn = pool.begin().await?;

    let mut device = Device::find_by_id(&mut txn, new_compiler.device_id, &user).await?;

    let mut sensors_and_alias = Vec::with_capacity(new_compiler.sensors.len());
    for sensor in new_compiler.sensors {
        let alias = sensor.alias.clone();
        let sensor = Sensor::new(&mut txn, sensor).await?;
        sensors_and_alias.push((sensor, alias));
    }
    let target = Target::find_by_id(&mut txn, new_compiler.target_id).await?;
    let (compiler, compilation) =
        dbg!(Compiler::new(&mut txn, &target, &sensors_and_alias, &device).await?);

    device
        .set_compiler_id(&mut txn, Some(compiler.id()))
        .await?;

    let view = CompilationView::new(&mut txn, compilation, &device).await?;

    txn.commit().await?;
    Ok(Json(view))
}
