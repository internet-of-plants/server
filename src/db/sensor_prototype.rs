pub mod builtin;

use crate::db::sensor::config_request::{ConfigRequest, NewConfigRequest};
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};

use super::sensor::config_request::ConfigRequestView;
use super::sensor::{Dependency, Include, Definition, Setup};
use super::sensor::measurement::Measurement;
use super::target::Target;

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
        targets: &[&Target],
    ) -> Result<Self> {
        let configuration_requests = prototype.configuration_requests(txn).await?;
        let mut configuration_requests_view = Vec::with_capacity(configuration_requests.len());
        for configuration_request in configuration_requests {
            configuration_requests_view
                .push(ConfigRequestView::new(txn, configuration_request, targets).await?);
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

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct SensorPrototypeId(pub i64);

impl SensorPrototypeId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Clone)]
pub struct SensorPrototype {
    id: SensorPrototypeId,
    name: String,
}

impl SensorPrototype {
    pub async fn new(
        txn: &mut Transaction<'_>,
        name: String,
        dependencies: Vec<Dependency>,
        includes: Vec<Include>,
        definitions: Vec<Definition>,
        setups: Vec<Setup>,
        measurements: Vec<Measurement>,
        new_config_requests: Vec<NewConfigRequest>,
    ) -> Result<Self> {
        let (sensor_prototype_id,) = sqlx::query_as::<_, (SensorPrototypeId,)>(
            "INSERT INTO sensor_prototypes (name) VALUES ($1) RETURNING id",
        )
        .bind(&name)
        .fetch_one(&mut *txn)
        .await?;
        for dep in &dependencies {
            sqlx::query(
                "INSERT INTO sensor_prototype_dependencies (dependency, sensor_prototype_id) VALUES ($1, $2)",
            )
            .bind(dep)
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
        for define in &definitions {
            sqlx::query(
                "INSERT INTO sensor_prototype_definitions (definition, sensor_prototype_id) VALUES ($1, $2)",
            )
            .bind(define)
            .bind(&sensor_prototype_id)
            .execute(&mut *txn)
            .await?;
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
        for measurement in &measurements {
            sqlx::query(
                "INSERT INTO sensor_prototype_measurements (name, value, ty, sensor_prototype_id, human_name, kind) VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind(&measurement.name)
            .bind(&measurement.value)
            .bind(&measurement.ty)
            .bind(&sensor_prototype_id)
            .bind(&measurement.human_name)
            .bind(&measurement.kind)
            .execute(&mut *txn)
            .await?;
        }
        for config_request in new_config_requests {
            ConfigRequest::new(
                &mut *txn,
                config_request.name,
                config_request.human_name,
                config_request.type_name,
                config_request.widget,
                sensor_prototype_id,
            )
            .await?;
        }
        Ok(Self {
            id: sensor_prototype_id,
            name,
        })
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        let prototype = sqlx::query_as("SELECT id, name FROM sensor_prototypes")
            .fetch_all(txn)
            .await?;
        Ok(prototype)
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: SensorPrototypeId) -> Result<Self> {
        let prototype = sqlx::query_as("SELECT id, name FROM sensor_prototypes WHERE id = $1")
            .bind(id)
            .fetch_one(txn)
            .await?;
        Ok(prototype)
    }

    pub fn id(&self) -> SensorPrototypeId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// A sensor should depend on N libraries (lib_dependencies param in platformio.ini)
    pub async fn dependencies(&self, txn: &mut Transaction<'_>) -> Result<Vec<Dependency>> {
        let list = sqlx::query_as(
            "SELECT dependency FROM sensor_prototype_dependencies WHERE sensor_prototype_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list.into_iter().map(|(text,)| text).collect())
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
        let list = sqlx::query_as(
            "SELECT definition FROM sensor_prototype_definitions WHERE sensor_prototype_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list.into_iter().map(|(text,)| text).collect())
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

    /// A sensor should execute N measurements and store them in the JSON
    pub async fn measurements(&self, txn: &mut Transaction<'_>) -> Result<Vec<Measurement>> {
        let list = sqlx::query_as(
            "SELECT human_name, name, value, ty, kind FROM sensor_prototype_measurements WHERE sensor_prototype_id = $1 ORDER BY id ASC",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        Ok(list)
    }

    /// A sensor should require 0-N configuration variables to be setup by the user
    pub async fn configuration_requests(
        &self,
        txn: &mut Transaction<'_>,
    ) -> Result<Vec<ConfigRequest>> {
        let list = sqlx::query_as(
            "SELECT id, name, human_name, type_id FROM config_requests WHERE sensor_prototype_id = $1",
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
        //    Config {
        //        ty: "Pin".to_owned(),
        //        name: "airTempAndHumidity".to_owned(),
        //        value: "Pin::D6".to_owned(),
        //    },
        //    Config {
        //        ty: "dht::Version".to_owned(),
        //        name: "dhtVersion".to_owned(),
        //        value: "dht::Version::DHT22".to_owned(),
        //    },
        //]);
        //let soil_resistivity = builtin::soil_resistivity().build(vec![Config {
        //    ty: "Pin".to_owned(),
        //    name: "soilResistivityPower".to_owned(),
        //    value: "Pin::D7".to_owned(),
        //}]);
        //let soil_temperature = builtin::soil_temperature().build(vec![Config {
        //    ty: "Pin".to_owned(),
        //    name: "soilTemperature".to_owned(),
        //    value: "Pin::D5".to_owned(),
        //}]);
        //let factory_reset_button = builtin::factory_reset_button().build(vec![Config {
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
