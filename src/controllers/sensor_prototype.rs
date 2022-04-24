use crate::db::sensor::config_request::{ConfigRequest, ConfigRequestId};
use crate::db::sensor::config_type::{ConfigType, WidgetKind};
use crate::db::sensor::Measurement;
use crate::db::sensor_prototype::{SensorPrototype, SensorPrototypeId};
use crate::db::target::TargetId;
use crate::prelude::*;
use controllers::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ConfigTypeView {
    name: String,
    widget: WidgetKind,
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

#[derive(Serialize)]
struct ConfigRequestView {
    id: ConfigRequestId,
    name: String,
    ty: ConfigTypeView,
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

#[derive(Serialize)]
struct SensorPrototypeView {
    id: SensorPrototypeId,
    name: String,
    dependencies: Vec<String>,
    includes: Vec<String>,
    definitions: Vec<String>,
    setups: Vec<String>,
    measurements: Vec<Measurement>,
    configuration_requests: Vec<ConfigRequestView>,
}

impl SensorPrototypeView {
    async fn new(
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

pub async fn index(pool: &'static Pool, _auth: Auth) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    let prototypes = SensorPrototype::list(&mut txn).await?;
    let mut views = Vec::with_capacity(prototypes.len());
    for prototype in prototypes {
        views.push(SensorPrototypeView::new(&mut txn, prototype, &[]).await?);
    }

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&views))
}

#[derive(Deserialize)]
pub struct TargetList {
    target: TargetId,
}

pub async fn find(
    sensor_prototype_id: SensorPrototypeId,
    pool: &'static Pool,
    _auth: Auth,
    target_ids: TargetList,
) -> Result<impl Reply> {
    let mut txn = pool.begin().await.map_err(Error::from)?;

    let prototype = SensorPrototype::find_by_id(&mut txn, sensor_prototype_id).await?;
    let view = SensorPrototypeView::new(&mut txn, prototype, &[target_ids.target]).await?;

    txn.commit().await.map_err(Error::from)?;
    Ok(warp::reply::json(&view))
}
