use crate::{
    extractor::User, Collection, CollectionId, CompilationView, Compiler, CompilerId, CompilerView,
    Device, DeviceConfig, DeviceId, NewCompiler, NewSensor, Organization, OrganizationId, Pool,
    Result, Sensor, Target, TargetId,
};
use axum::extract::{Extension, Json, Query};
use serde::Deserialize;
use std::collections::HashMap;

pub async fn new(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(new_compiler): Json<NewCompiler>,
) -> Result<Json<CompilationView>> {
    let mut txn = pool.begin().await?;

    let mut device = match new_compiler.device_id() {
        Some(device_id) => Some(Device::find_by_id(&mut txn, device_id, &user).await?),
        None => None,
    };
    let mut collection =
        Collection::find_by_id(&mut txn, new_compiler.collection_id(), &user).await?;
    let organization = collection.organization(&mut txn).await?;

    let target = Target::find_by_id(&mut txn, new_compiler.target_id()).await?;

    let sensors = NewSensor::sort(&mut txn, new_compiler.sensors(), &[&target]).await?;

    let mut sensor_by_local_pk: HashMap<u64, Sensor> = HashMap::new();
    let mut sensors_and_alias = Vec::new();
    for (index, mut sensor) in sensors.into_iter().enumerate() {
        let alias = sensor.alias().to_owned();
        let local_pk = sensor.local_pk();

        sensor
            .normalize(&mut txn, &mut sensor_by_local_pk, &[&target])
            .await?;

        let sensor = Sensor::new(&mut txn, sensor, index as i64, &[&target]).await?;
        sensor_by_local_pk.insert(local_pk, sensor.clone());
        sensors_and_alias.push((sensor, alias));
    }

    let mut device_configs = Vec::new();
    for config in new_compiler.device_configs() {
        let config = DeviceConfig::new(&mut txn, config.clone(), &organization).await?;
        device_configs.push(config);
    }

    // TODO: encrypt secret
    let (_compiler, compilation) = Compiler::new(
        &mut txn,
        &target,
        sensors_and_alias,
        device_configs,
        &mut collection,
        &mut device,
    )
    .await?;

    let view = CompilationView::new(compilation);

    txn.commit().await?;
    Ok(Json(view))
}

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
enum Id {
    CollectionId { collection_id: CollectionId },
    DeviceId { device_id: DeviceId },
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetRequest {
    id: Id,
    compiler_id: CompilerId,
}

pub async fn set(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Json(request): Json<SetRequest>,
) -> Result<Json<CompilationView>> {
    let mut txn = pool.begin().await?;

    let mut collection = match &request.id {
        Id::CollectionId { collection_id } => {
            Collection::find_by_id(&mut txn, *collection_id, &user).await?
        }
        Id::DeviceId { device_id } => {
            let device = Device::find_by_id(&mut txn, *device_id, &user).await?;
            device.collection(&mut txn).await?
        }
    };
    let organization = collection.organization(&mut txn).await?;
    let compiler = Compiler::find_by_id(&mut txn, &organization, request.compiler_id).await?;

    if let Some(col) = compiler.collection(&mut txn).await? {
        for device in collection.devices(&mut txn).await? {
            Device::update_collection(&mut txn, device.id(), &col).await?;
        }
        collection.delete(&mut txn).await?;
    } else if let Id::DeviceId { device_id } = request.id {
        if Device::from_collection(&mut txn, &collection).await?.len() == 1 {
            collection.set_compiler(&mut txn, Some(&compiler)).await?;
        } else {
            let mut device = Device::find_by_id(&mut txn, device_id, &user).await?;
            collection = Collection::new(&mut txn, device.name().to_owned(), &organization).await?;
            collection.set_compiler(&mut txn, Some(&compiler)).await?;
            device.set_collection(&mut txn, &collection).await?;
        }
    } else if let Id::CollectionId { collection_id } = request.id {
        let mut collection = Collection::find_by_id(&mut txn, collection_id, &user).await?;
        collection.set_compiler(&mut txn, Some(&compiler)).await?;
    }
    let compilation = CompilationView::new(compiler.latest_compilation(&mut txn).await?);

    txn.commit().await?;
    Ok(Json(compilation))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    target_id: TargetId,
    organization_id: OrganizationId,
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    User(user): User,
    Query(request): Query<ListRequest>,
) -> Result<Json<Vec<CompilerView>>> {
    let mut txn = pool.begin().await?;

    let organization = Organization::find_by_id(&mut txn, request.organization_id, &user).await?;
    let target = Target::find_by_id(&mut txn, request.target_id).await?;
    let compilers = Compiler::list_for_target(&mut txn, &organization, &target).await?;

    let mut views = Vec::with_capacity(compilers.len());
    for compiler in compilers {
        // Compilers without collections aren't helpful
        if let Some(collection) = compiler.collection(&mut txn).await? {
            if !Device::from_collection(&mut txn, &collection)
                .await?
                .is_empty()
            {
                views.push(CompilerView::new(&mut txn, compiler).await?);
            }
        }
    }

    txn.commit().await?;
    Ok(Json(views))
}
