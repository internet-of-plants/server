use crate::db::sensor::config_request::{ConfigRequest, ConfigRequestId};
use crate::db::sensor::config_type::{ConfigType, WidgetKind};
use crate::db::sensor::Measurement;
use crate::db::sensor_prototype::{SensorPrototype, SensorPrototypeId};
use crate::db::target::TargetId;
use crate::extractor::Authorization;
use crate::prelude::*;
use axum::extract::{Extension, Json, Path, Query};
use controllers::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConfigTypeView {
    pub name: String,
    pub widget: WidgetKind,
}

impl ConfigTypeView {
    async fn new(
        txn: &mut Transaction<'_>,
        ty: ConfigType,
        target_ids: &[TargetId],
    ) -> Result<Self> {
        Ok(Self {
            name: ty.name().to_owned(),
            widget: ty.widget(txn, target_ids).await?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConfigRequestView {
    pub id: ConfigRequestId,
    pub name: String,
    pub ty: ConfigTypeView,
}

impl ConfigRequestView {
    async fn new(
        txn: &mut Transaction<'_>,
        request: ConfigRequest,
        target_ids: &[TargetId],
    ) -> Result<Self> {
        let ty = request.ty(txn).await?;
        Ok(Self {
            id: request.id(),
            name: request.name().to_owned(),
            ty: ConfigTypeView::new(txn, ty, target_ids).await?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SensorPrototypeView {
    pub id: SensorPrototypeId,
    pub name: String,
    pub dependencies: Vec<String>,
    pub includes: Vec<String>,
    pub definitions: Vec<String>,
    pub setups: Vec<String>,
    pub measurements: Vec<Measurement>,
    pub configuration_requests: Vec<ConfigRequestView>,
}

impl SensorPrototypeView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        prototype: SensorPrototype,
        target_ids: &[TargetId],
    ) -> Result<Self> {
        let configuration_requests = prototype.configuration_requests(txn).await?;
        let mut configuration_requests_view = Vec::with_capacity(configuration_requests.len());
        for configuration_request in configuration_requests {
            configuration_requests_view
                .push(ConfigRequestView::new(txn, configuration_request, target_ids).await?);
        }
        Ok(Self {
            id: prototype.id(),
            name: prototype.name().to_owned(),
            dependencies: prototype.dependencies(txn).await?,
            includes: prototype.includes(txn).await?,
            definitions: prototype.definitions(txn).await?,
            setups: prototype.setups(txn).await?,
            measurements: prototype.measurements(txn).await?,
            configuration_requests: configuration_requests_view,
        })
    }
}

pub async fn list(
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<Vec<SensorPrototypeView>>> {
    let mut txn = pool.begin().await?;

    let prototypes = SensorPrototype::list(&mut txn).await?;
    let mut views = Vec::with_capacity(prototypes.len());
    for prototype in prototypes {
        views.push(SensorPrototypeView::new(&mut txn, prototype, &[]).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}

pub async fn list_for_target(
    Path(target_id): Path<TargetId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
) -> Result<Json<Vec<SensorPrototypeView>>> {
    let mut txn = pool.begin().await?;

    let prototypes = SensorPrototype::list(&mut txn).await?;
    let mut views = Vec::with_capacity(prototypes.len());
    for prototype in prototypes {
        views.push(SensorPrototypeView::new(&mut txn, prototype, &[target_id]).await?);
    }

    txn.commit().await?;
    Ok(Json(views))
}

#[derive(Deserialize)]
pub struct RequestTarget {
    target_id: Option<TargetId>,
}

pub async fn find(
    Path(sensor_prototype_id): Path<SensorPrototypeId>,
    Extension(pool): Extension<&'static Pool>,
    Authorization(_auth): Authorization,
    Query(target): Query<RequestTarget>,
) -> Result<Json<SensorPrototypeView>> {
    let mut txn = pool.begin().await?;

    let prototype = SensorPrototype::find_by_id(&mut txn, sensor_prototype_id).await?;
    let view = SensorPrototypeView::new(
        &mut txn,
        prototype,
        &target.target_id.map_or(vec![], |id| vec![id]),
    )
    .await?;

    txn.commit().await?;
    Ok(Json(view))
}
