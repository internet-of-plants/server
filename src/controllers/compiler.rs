use crate::{
    extractor::User, Collection, CollectionId, CompilationView, Compiler, CompilerId, CompilerView,
    Device, DeviceConfig, DeviceId, Error, NewCompiler, NewSensor, Organization, OrganizationId,
    Pool, Result, Sensor, SensorConfigRequest, SensorPrototype, SensorWidgetKindView, Target,
    TargetId, Val,
};
use axum::extract::{Extension, Json, Query};
use serde::Deserialize;
use std::collections::{HashMap, HashSet, VecDeque};

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

    let sensor_by_local_pk: HashMap<usize, NewSensor> = new_compiler
        .sensors()
        .iter()
        .map(|s| (s.local_pk(), s.clone()))
        .collect();
    let mut graph: HashMap<usize, HashSet<usize>> = HashMap::new();

    let target = Target::find_by_id(&mut txn, new_compiler.target_id()).await?;

    for sensor in sensor_by_local_pk.values() {
        let value = graph.entry(sensor.local_pk()).or_default();
        for config in sensor.configs() {
            let request = SensorConfigRequest::find_by_id(&mut txn, config.request_id()).await?;
            let ty = request.ty(&mut txn).await?;
            if let SensorWidgetKindView::Sensor(_) = ty.widget(&mut txn, &[&target]).await? {
                match config.value {
                    Val::String(_) | Val::Map(_) => unimplemented!(),
                    Val::Number(number) => {
                        value.insert(number);
                    }
                }
            }
        }
    }

    let mut sorted_sensors = Vec::new();
    let mut work_queue: VecDeque<usize> = graph
        .iter()
        .filter(|(_, c)| c.is_empty())
        .map(|(p, _)| *p)
        .collect();

    while let Some(local_pk) = work_queue.pop_front() {
        let sensor = sensor_by_local_pk.get(&local_pk).ok_or_else(|| {
            Error::NewSensorReferencedDoesntExist(
                local_pk,
                sensor_by_local_pk.iter().map(|(k, _)| *k).collect(),
            )
        })?;
        sorted_sensors.push(sensor.clone());

        graph.remove(&sensor.local_pk());

        for (parent, children) in graph.iter_mut() {
            children.retain(|el| *el != sensor.local_pk());
            if children.is_empty() {
                work_queue.push_back(*parent);
            }
        }
    }
    let sorted_sensors: Vec<(usize, NewSensor)> = sorted_sensors.into_iter().enumerate().collect();

    let mut sensors_and_alias = Vec::new();
    let mut sensor_by_local_pk: HashMap<usize, Sensor> = HashMap::new();
    for (index, mut sensor) in sorted_sensors.clone() {
        let alias = sensor.alias().to_owned();
        let prototype = SensorPrototype::find_by_id(&mut txn, sensor.prototype_id()).await?;

        for config in sensor.configs_mut() {
            let request = SensorConfigRequest::find_by_id(&mut txn, config.request_id()).await?;
            let ty = request.ty(&mut txn).await?;
            if let SensorWidgetKindView::Sensor(dependency_prototype_id) =
                ty.widget(&mut txn, &[&target]).await?
            {
                let dependency_prototype =
                    SensorPrototype::find_by_id(&mut txn, dependency_prototype_id).await?;
                if let Some(variable_name) = dependency_prototype.variable_name() {
                    match config.value {
                        Val::String(_) | Val::Map(_) => unimplemented!(),
                        Val::Number(local_pk) => {
                            let index = sorted_sensors
                                .iter()
                                .find(|(_, s)| s.local_pk() == local_pk)
                                .unwrap()
                                .0;
                            let child_sensor =
                                sensor_by_local_pk.get(&local_pk).ok_or_else(|| {
                                    Error::NewSensorReferencedDoesntExist(
                                        local_pk,
                                        sensor_by_local_pk.iter().map(|(k, _)| *k).collect(),
                                    )
                                })?;

                            if child_sensor.prototype_id() == dependency_prototype_id {
                                for def in &prototype.definitions(&mut txn).await? {
                                    for sensor_referenced in def.sensors_referenced() {
                                        if sensor_referenced.sensor_name()
                                            != dependency_prototype.name()
                                        {
                                            continue;
                                        }

                                        config.value =
                                            Val::String(format!("{}{index}", variable_name));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let local_pk = sensor.local_pk();
        let sensor = Sensor::new(&mut txn, sensor, index as i64).await?;
        sensor_by_local_pk.insert(local_pk, sensor.clone());
        sensors_and_alias.push((sensor, alias));
    }
    let mut device_configs = Vec::new();
    for config in new_compiler.device_configs() {
        let config = DeviceConfig::new(&mut txn, config.clone(), &organization).await?;
        device_configs.push(config);
    }
    // TODO: encrypt secrets
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
            if Device::from_collection(&mut txn, &collection).await?.len() > 0 {
                views.push(CompilerView::new(&mut txn, compiler).await?);
            }
        }
    }

    txn.commit().await?;
    Ok(Json(views))
}
