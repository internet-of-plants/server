use crate::{
    Definition, Dependency, Include, NewSensorConfigRequest, Result, SensorConfigRequest,
    SensorConfigRequestView, SensorMeasurement, SensorPrototypeDefinitionId, Setup, Target,
    Transaction, UnauthenticatedAction,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};

#[derive(Getters, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SensorPrototypeView {
    #[copy]
    id: SensorPrototypeId,
    name: String,
    dependencies: Vec<Dependency>,
    includes: Vec<Include>,
    definitions: Vec<Definition>,
    setups: Vec<Setup>,
    unauthenticated_actions: Vec<UnauthenticatedAction>,
    measurements: Vec<SensorMeasurement>,
    configuration_requests: Vec<SensorConfigRequestView>,
    variable_name: Option<String>,
}

impl SensorPrototypeView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        prototype: SensorPrototype,
        targets: &[&Target],
    ) -> Result<Self> {
        let configuration_requests = prototype.configuration_requests(txn).await?;
        let mut configuration_requests_view = Vec::with_capacity(configuration_requests.len());
        for configuration_request in configuration_requests {
            configuration_requests_view
                .push(SensorConfigRequestView::new(txn, configuration_request, targets).await?);
        }
        Ok(Self {
            id: prototype.id(),
            name: prototype.name().to_owned(),
            variable_name: prototype.variable_name().to_owned(),
            dependencies: prototype.dependencies(txn).await?,
            includes: prototype.includes(txn).await?,
            definitions: prototype.definitions(txn).await?,
            setups: prototype.setups(txn).await?,
            unauthenticated_actions: prototype.unauthenticated_actions(txn).await?,
            measurements: prototype.measurements(txn).await?,
            configuration_requests: configuration_requests_view,
        })
    }
}

#[id]
pub struct SensorPrototypeId;

#[derive(sqlx::FromRow, Getters, Serialize, Deserialize, Debug, Clone)]
pub struct SensorPrototype {
    #[copy]
    id: SensorPrototypeId,
    name: String,
    variable_name: Option<String>,
}

impl SensorPrototype {
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: impl Into<String>,
        variable_name: impl Into<Option<&str>>,
        dependencies: Vec<Dependency>,
        includes: Vec<Include>,
        definitions: Vec<impl Into<Definition>>,
        setups: Vec<Setup>,
        unauthenticated_actions: Vec<UnauthenticatedAction>,
        measurements: Vec<SensorMeasurement>,
        new_config_requests: Vec<NewSensorConfigRequest>,
    ) -> Result<Self> {
        let (name, variable_name) = (name.into(), variable_name.into());
        let (sensor_prototype_id,) = sqlx::query_as::<_, (SensorPrototypeId,)>(
            "INSERT INTO sensor_prototypes (name, variable_name) VALUES ($1, $2) RETURNING id",
        )
        .bind(&name)
        .bind(&variable_name)
        .fetch_one(&mut *txn)
        .await?;
        let sensor_prototype = Self {
            id: sensor_prototype_id,
            variable_name: variable_name.map(ToOwned::to_owned),
            name,
        };

        for dep in &dependencies {
            sqlx::query(
                "INSERT INTO sensor_prototype_dependencies (repo_url, branch, sensor_prototype_id) VALUES ($1, $2, $3)",
            )
            .bind(dep.repo_url())
            .bind(dep.branch())
            .bind(&sensor_prototype_id)
            .execute(&mut *txn)
            .await?;
        }
        for include in &includes {
            sqlx::query(
                "INSERT INTO sensor_prototype_includes (include, sensor_prototype_id) VALUES ($1, $2)",
            )
            .bind(include)
            .bind(&sensor_prototype_id)
            .execute(&mut *txn)
            .await?;
        }
        for define in definitions {
            let define = define.into();
            let (sensor_prototype_definition_id,) = sqlx::query_as::<_, (SensorPrototypeDefinitionId,)>(
                "INSERT INTO sensor_prototype_definitions (line, sensor_prototype_id) VALUES ($1, $2) RETURNING id",
            )
            .bind(define.line())
            .bind(&sensor_prototype_id)
            .fetch_one(&mut *txn)
            .await?;
            for sensor_referenced in define.sensors_referenced() {
                sqlx::query(
                    "INSERT INTO sensor_prototype_definition_sensors_referenced (sensor_name, request_name, sensor_prototype_definition_id) VALUES ($1, $2, $3)",
                )
                .bind(sensor_referenced.sensor_name())
                .bind(sensor_referenced.request_name())
                .bind(&sensor_prototype_definition_id)
                .execute(&mut *txn)
                .await?;
            }
        }
        for setup in &setups {
            sqlx::query(
                "INSERT INTO sensor_prototype_setups (setup, sensor_prototype_id) VALUES ($1, $2)",
            )
            .bind(setup)
            .bind(&sensor_prototype_id)
            .execute(&mut *txn)
            .await?;
        }
        for unauthenticated_action in &unauthenticated_actions {
            sqlx::query(
                "INSERT INTO sensor_prototype_unauthenticated_actions (unauthenticated_action, sensor_prototype_id) VALUES ($1, $2)",
            )
            .bind(unauthenticated_action)
            .bind(&sensor_prototype_id)
            .execute(&mut *txn)
            .await?;
        }
        for measurement in &measurements {
            sqlx::query(
                "INSERT INTO sensor_prototype_measurements (name, value, ty, sensor_prototype_id, human_name, kind) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(measurement.name())
            .bind(measurement.value())
            .bind(measurement.ty())
            .bind(&sensor_prototype_id)
            .bind(measurement.human_name())
            .bind(measurement.kind())
            .execute(&mut *txn)
            .await?;
        }
        for config_request in new_config_requests {
            SensorConfigRequest::new(
                &mut *txn,
                config_request.name().clone(),
                config_request.human_name().clone(),
                config_request.type_name().clone(),
                config_request.widget().clone(),
                &sensor_prototype,
            )
            .await?;
        }
        Ok(sensor_prototype)
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        let prototype = sqlx::query_as("SELECT id, name, variable_name FROM sensor_prototypes")
            .fetch_all(txn)
            .await?;
        Ok(prototype)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: SensorPrototypeId) -> Result<Self> {
        let prototype =
            sqlx::query_as("SELECT id, name, variable_name FROM sensor_prototypes WHERE id = $1")
                .bind(id)
                .fetch_one(txn)
                .await?;
        Ok(prototype)
    }

    /// A sensor should depend on N libraries (lib_dependencies param in platformio.ini)
    pub async fn dependencies(&self, txn: &mut Transaction<'_>) -> Result<Vec<Dependency>> {
        let list = sqlx::query_as(
            "SELECT repo_url, branch FROM sensor_prototype_dependencies WHERE sensor_prototype_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list)
    }

    /// A sensor should import N libraries (#include expressions)
    pub async fn includes(&self, txn: &mut Transaction<'_>) -> Result<Vec<Include>> {
        let list = sqlx::query_as(
            "SELECT include FROM sensor_prototype_includes WHERE sensor_prototype_id = $1 ORDER BY include ASC",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list.into_iter().map(|(text,)| text).collect())
    }

    /// A sensor should declare N variables to hold its state
    pub async fn definitions(&self, txn: &mut Transaction<'_>) -> Result<Vec<Definition>> {
        let defs: Vec<(SensorPrototypeDefinitionId, String)> = sqlx::query_as(
            "SELECT id, line FROM sensor_prototype_definitions WHERE sensor_prototype_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        let mut list = Vec::with_capacity(defs.len());
        for (id, line) in defs {
            list.push(Definition::new(line, sqlx::query_as("SELECT sensor_name, request_name FROM sensor_prototype_definition_sensors_referenced WHERE sensor_prototype_definition_id = $1").bind(&id).fetch_all(&mut *txn).await?));
        }
        Ok(list)
    }

    /// A sensor should have code to setup itself
    pub async fn setups(&self, txn: &mut Transaction<'_>) -> Result<Vec<Setup>> {
        let list = sqlx::query_as(
            "SELECT setup FROM sensor_prototype_setups WHERE sensor_prototype_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list.into_iter().map(|(text,)| text).collect())
    }

    /// A sensor may have unauthenticated actions to execute
    pub async fn unauthenticated_actions(&self, txn: &mut Transaction<'_>) -> Result<Vec<Setup>> {
        let list = sqlx::query_as(
            "SELECT unauthenticated_action FROM sensor_prototype_unauthenticated_actions WHERE sensor_prototype_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list.into_iter().map(|(text,)| text).collect())
    }

    /// A sensor should execute N measurements and store them in the JSON
    pub async fn measurements(&self, txn: &mut Transaction<'_>) -> Result<Vec<SensorMeasurement>> {
        let list = sqlx::query_as(
            "SELECT human_name, name, value, ty, kind FROM sensor_prototype_measurements WHERE sensor_prototype_id = $1 ORDER BY id ASC",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list)
    }

    /// A sensor should require 0-N configuration variables to be defined by the user
    pub async fn configuration_requests(
        &self,
        txn: &mut Transaction<'_>,
    ) -> Result<Vec<SensorConfigRequest>> {
        let list = sqlx::query_as(
            "SELECT id, name, human_name, type_id FROM sensor_config_requests WHERE sensor_prototype_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn all_sensors() {
        //let dht = builtin::dht().build(vec![
        //    SensorConfig {
        //        ty: "Pin".to_owned(),
        //        name: "airTempAndHumidity".to_owned(),
        //        value: "Pin::D6".to_owned(),
        //    },
        //    SensorConfig {
        //        ty: "dht::Version".to_owned(),
        //        name: "dhtVersion".to_owned(),
        //        value: "dht::Version::DHT22".to_owned(),
        //    },
        //]);
        //let soil_resistivity = builtin::soil_resistivity().build(vec![SensorConfig {
        //    ty: "Pin".to_owned(),
        //    name: "soilResistivityPower".to_owned(),
        //    value: "Pin::D7".to_owned(),
        //}]);
        //let soil_temperature = builtin::soil_temperature().build(vec![SensorConfig {
        //    ty: "Pin".to_owned(),
        //    name: "soilTemperature".to_owned(),
        //    value: "Pin::D5".to_owned(),
        //}]);
        //let factory_reset_button = builtin::factory_reset_button().build(vec![SensorConfig {
        //    ty: "Pin".to_owned(),
        //    name: "factoryResetButton".to_owned(),
        //    value: "Pin::D1".to_owned(),
        //}]);
        //assert_eq!(
        //    Sensors(vec![
        //        factory_reset_button,
        //        soil_resistivity,
        //        soil_temperature,
        //        dht,
        //    ])
        //    .compile(),
        //    include_str!("../../test/main.cpp")
        //);
    }
}
