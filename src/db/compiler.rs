use crate::{
    Collection, CollectionId, Compilation, Device, DeviceConfig, DeviceConfigView, DeviceId,
    DeviceWidgetKind, Error, FirmwareView, NewDeviceConfig, NewSensor, Organization, Result,
    Sensor, SensorConfigRequest, SensorView, Target, TargetId, TargetView, Transaction,
};
use derive::id;
use handlebars::Handlebars;
use random_color::RandomColor;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use derive_get::Getters;

#[derive(Getters, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewCompiler {
    #[copy]
    collection_id: CollectionId,
    #[copy]
    device_id: Option<DeviceId>,
    #[copy]
    target_id: TargetId,
    device_configs: Vec<NewDeviceConfig>,
    sensors: Vec<NewSensor>,
}

#[derive(Getters, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompilerView {
    #[copy]
    id: CompilerId,
    sensors: Vec<SensorView>,
    device_configs: Vec<DeviceConfigView>,
    collection_name: String,
    #[copy]
    devices_count: usize,
    target: TargetView,
    latest_firmware: FirmwareView,
}

impl CompilerView {
    pub async fn new(txn: &mut Transaction<'_>, compiler: Compiler) -> Result<Self> {
        let collection = match compiler.collection(txn).await? {
            Some(col) => col,
            None => return Err(Error::NoCollectionForCompiler(compiler.id())),
        };
        let devices = Device::from_collection(txn, &collection).await?;
        let sensors = compiler.sensors(txn).await?;

        let mut device_configs = Vec::new();
        for config in compiler.device_configs(txn).await? {
            device_configs.push(DeviceConfigView::new(txn, config).await?);
        }

        let target = compiler.target(txn).await?;
        let target = TargetView::new(txn, target).await?;

        let latest_compilation = compiler.latest_compilation(txn).await?;
        let latest_firmware = latest_compilation.firmware(txn).await?;
        let latest_firmware = FirmwareView::new(latest_firmware);

        Ok(Self {
            id: compiler.id(),
            sensors,
            target,
            latest_firmware,
            device_configs,
            devices_count: devices.len(),
            collection_name: collection.name().to_owned(),
        })
    }
}

#[id]
pub struct CompilerId;

impl CompilerId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Compiler {
    id: CompilerId,
    target_id: TargetId,
}

impl Compiler {
    pub async fn new(
        txn: &mut Transaction<'_>,
        target: &Target,
        mut sensors_and_alias: Vec<(Sensor, String)>,
        mut device_configs: Vec<DeviceConfig>,
        collection: &mut Collection,
        device: &mut Option<Device>,
    ) -> Result<(Self, Compilation)> {
        let organization = collection.organization(txn).await?;

        sensors_and_alias.dedup_by_key(|(s, _)| s.id());
        device_configs.dedup_by_key(|c| c.id());

        let id: Option<(CompilerId,)> = sqlx::query_as(
            "SELECT compilers.id
             FROM (SELECT COUNT(sensor_id) as count, compiler_id
                   FROM (SELECT sbt.sensor_id, sbt.compiler_id
                         FROM sensor_belongs_to_compiler as sbt
                         WHERE sbt.sensor_id = ANY($1)) as s_bt_c
                   GROUP BY s_bt_c.compiler_id) as sensor,
                  (SELECT COUNT(config_id) as count, compiler_id
                   FROM (SELECT dbt.config_id, dbt.compiler_id
                         FROM device_config_belongs_to_compiler as dbt
                         WHERE dbt.config_id = ANY($2)) as d_bt_c
                   GROUP BY d_bt_c.compiler_id) as device
             INNER JOIN compilers ON compilers.id = device.compiler_id
             WHERE device.compiler_id = sensor.compiler_id
                   AND compilers.target_id = $5
                   AND compilers.organization_id = $6
             GROUP BY compilers.id, device.count, sensor.count
             HAVING sensor.count = $3 AND device.count = $4",
        )
        .bind(
            &sensors_and_alias
                .iter()
                .map(|s| s.0.id())
                .collect::<Vec<_>>(),
        )
        .bind(&device_configs.iter().map(|s| s.id()).collect::<Vec<_>>())
        .bind(&(sensors_and_alias.len() as i64))
        .bind(&(device_configs.len() as i64))
        .bind(target.id())
        .bind(organization.id())
        .fetch_optional(&mut *txn)
        .await?;

        let mut should_compile = false;
        let id = if let Some((id,)) = id {
            id
        } else {
            should_compile = true;

            let (id,): (CompilerId,) = sqlx::query_as(
                "INSERT INTO compilers (target_id, organization_id) VALUES ($1, $2) RETURNING id",
            )
            .bind(target.id())
            .bind(organization.id())
            .fetch_one(&mut *txn)
            .await?;

            for (sensor, alias) in sensors_and_alias {
                let color = RandomColor::new().to_hsl_string();
                sqlx::query(
                    "INSERT INTO sensor_belongs_to_compiler (sensor_id, compiler_id, alias, color) VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING",
                )
                    .bind(sensor.id())
                    .bind(&id)
                    .bind(&alias)
                    .bind(&color)
                    .execute(&mut *txn)
                    .await?;
            }

            for device_config in device_configs {
                sqlx::query(
                    "INSERT INTO device_config_belongs_to_compiler (config_id, compiler_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                    .bind(device_config.id())
                    .bind(&id)
                    .execute(&mut *txn)
                    .await?;
            }
            id
        };

        let compiler = Self {
            id,
            target_id: target.id(),
        };

        if let Some(col) = compiler.collection(txn).await? {
            match device {
                Some(device) => device.set_collection(txn, &col).await?,
                None => {
                    for mut device in Device::from_collection(txn, collection).await? {
                        device.set_collection(txn, &col).await?;
                    }
                }
            }
            *collection = col;
        } else if let Some(device) = device {
            if Device::from_collection(txn, &collection).await?.len() == 1 {
                collection.set_compiler(txn, Some(&compiler)).await?;
            } else {
                let mut col = Collection::new(txn, device.name().to_owned(), &organization).await?;
                col.set_compiler(txn, Some(&compiler)).await?;
                device.set_collection(txn, collection).await?;
            }
        } else {
            collection.set_compiler(txn, Some(&compiler)).await?;
        }

        let compilation = if should_compile {
            compiler.compile(txn).await?
        } else {
            compiler.latest_compilation(txn).await?
        };
        Ok((compiler, compilation))
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        organization: &Organization,
        id: CompilerId,
    ) -> Result<Self> {
        let comp = sqlx::query_as(
            "SELECT compilers.id, compilers.target_id
             FROM compilers
             INNER JOIN collections ON collections.compiler_id = compilers.id
             INNER JOIN collection_belongs_to_organization bt ON bt.collection_id = collections.id
             INNER JOIN organizations ON organizations.id = bt.organization_id
             WHERE compilers.id = $1 AND organizations.id = $2",
        )
        .bind(id)
        .bind(organization.id())
        .fetch_one(txn)
        .await?;
        Ok(comp)
    }

    pub async fn list_for_target(
        txn: &mut Transaction<'_>,
        organization: &Organization,
        target: &Target,
    ) -> Result<Vec<Self>> {
        let comps = sqlx::query_as(
            "SELECT compilers.id, compilers.target_id
             FROM compilers
             WHERE compilers.target_id = $1 AND compilers.organization_id = $2",
        )
        .bind(target.id())
        .bind(organization.id())
        .fetch_all(txn)
        .await?;
        Ok(comps)
    }

    pub async fn find_by_compilation(
        txn: &mut Transaction<'_>,
        compilation: &Compilation,
    ) -> Result<Self> {
        let comp = sqlx::query_as(
            "SELECT compilers.id, compilers.target_id
             FROM compilers
             INNER JOIN compilations ON compilations.compiler_id = compilers.id
             WHERE compilers.id = $1 AND compilations.id = $2",
        )
        .bind(compilation.compiler_id())
        .bind(compilation.id())
        .fetch_one(txn)
        .await?;
        Ok(comp)
    }

    pub fn id(&self) -> CompilerId {
        self.id
    }

    pub async fn target(&self, txn: &mut Transaction<'_>) -> Result<Target> {
        Target::find_by_id(txn, self.target_id).await
    }

    pub async fn sensors(&self, txn: &mut Transaction<'_>) -> Result<Vec<SensorView>> {
        SensorView::list_for_compiler(txn, self).await
    }

    pub async fn compile(&self, txn: &mut Transaction<'_>) -> Result<Compilation> {
        let sensors = self.sensors(txn).await?;

        let device_configs_raw = self.device_configs(txn).await?;
        let mut device_configs = Vec::with_capacity(device_configs_raw.len());
        // TODO: properly use device config
        for config in &device_configs_raw {
            let request = config.request(txn).await?;
            let ty = request.ty(txn).await?;
            // TODO: validate SSID and PSK sizes and Timezone
            match ty.widget() {
                DeviceWidgetKind::SSID => device_configs.push(
                    format!("constexpr static char SSID_ROM_RAW[] IOP_ROM = \"{0}\";\nstatic const iop::StaticString SSID = reinterpret_cast<const __FlashStringHelper*>(SSID_ROM_RAW);", config.value().replace('"', "\\\""))
                ),
                DeviceWidgetKind::PSK => device_configs.push(
                    format!("constexpr static char PSK_ROM_RAW[] IOP_ROM = \"{0}\";\nstatic const iop::StaticString PSK = reinterpret_cast<const __FlashStringHelper*>(PSK_ROM_RAW);", config.value().replace('"', "\\\""))
                ),
                DeviceWidgetKind::Timezone => device_configs.push(
                    format!("constexpr static int8_t timezone = {0};", config.value().parse::<i8>().map_err(|err| Error::InvalidTimezone(err, config.value().clone()))?)
                )
            }
        }
        let device_configs = device_configs.join("\n");

        let mut lib_deps = Vec::new();
        let mut includes = Vec::new();
        let mut definitions = Vec::new();
        let mut measurements = Vec::new();
        let mut setups = Vec::new();
        let mut unauthenticated_actions = Vec::new();
        let mut configs = Vec::with_capacity(sensors.len());
        for (index, sensor) in sensors.iter().enumerate() {
            let prototype = sensor.prototype();
            lib_deps.extend(prototype.dependencies().clone());
            includes.extend(
                prototype
                    .includes()
                    .into_iter()
                    .map(|name| format!("#include <{name}>")),
            );
            let mut local_definitions = Vec::with_capacity(prototype.definitions().len());
            for definition in prototype.definitions() {
                let reg = Handlebars::new();
                let mut map = HashMap::new();
                map.insert("index".to_owned(), index.to_string());
                'outer: for sensor_referenced in definition.sensors_referenced() {
                    for other_sensor in &sensors {
                        if other_sensor.name() == sensor_referenced.sensor_name() {
                            for c in sensor.configurations() {
                                let req =
                                    SensorConfigRequest::find_by_id(txn, c.request_id()).await?;
                                if req.name() == sensor_referenced.request_name() {
                                    map.insert(
                                        sensor_referenced.request_name().clone(),
                                        c.value().clone(),
                                    );
                                    continue 'outer;
                                }
                            }
                        }
                    }
                }
                local_definitions
                    .push(reg.render_template(definition.line(), &serde_json::to_value(&map)?)?);
            }
            definitions.push(local_definitions.join("\n"));
            measurements.push(
                prototype
                    .measurements()
                    .into_iter()
                    .map(|m| {
                        let reg = Handlebars::new();
                        let name = reg.render_template(m.name(), &json!({ "index": index }))?;
                        let value = reg.render_template(m.value(), &json!({ "index": index }))?;
                        Ok(format!("doc[\"{}\"] = {}", name, value))
                    })
                    .collect::<Result<Vec<String>>>()?
                    .join("\n    "),
            );
            setups.extend(
                prototype
                    .setups()
                    .into_iter()
                    .map(|setup| {
                        let reg = Handlebars::new();
                        reg.render_template(setup, &json!({ "index": index }))
                    })
                    .collect::<Result<Vec<String>, _>>()?,
            );
            unauthenticated_actions.extend(
                prototype
                    .unauthenticated_actions()
                    .iter()
                    .map(|unauthenticated_action| {
                        let reg = Handlebars::new();
                        reg.render_template(unauthenticated_action, &json!({ "index": index }))
                    })
                    .collect::<Result<Vec<String>, _>>()?,
            );

            for c in sensor.configurations() {
                let req = SensorConfigRequest::find_by_id(txn, c.request_id()).await?;
                let reg = Handlebars::new();
                let ty = req.ty(txn).await?;
                if let Some(type_name) = ty.name() {
                    let name = reg.render_template(req.name(), &json!({ "index": index }))?;
                    configs.push((
                        req.name().clone(),
                        format!("static const {} {} = {};", type_name, name, c.value()),
                    ));
                }
            }
        }
        lib_deps.dedup();
        lib_deps.sort_unstable();

        includes.dedup();
        includes.sort_unstable();

        measurements.sort_unstable();

        configs.sort_by(|a, b| a.0.cmp(&b.0));

        setups.sort_unstable();
        unauthenticated_actions.sort_unstable();

        // TODO: properly use device config
        for config in &device_configs_raw {
            let request = config.request(txn).await?;
            let ty = request.ty(txn).await?;
            // TODO: validate SSID and PSK sizes
            match ty.widget() {
                DeviceWidgetKind::SSID => setups.insert(
                    0,
                    "loop.setAccessPointCredentials(config::SSID, config::PSK);".to_owned(),
                ),
                DeviceWidgetKind::PSK => {}
                DeviceWidgetKind::Timezone => {
                    setups.insert(0, "loop.setTimezone(config::timezone);".to_owned())
                }
            }
        }

        definitions.sort_unstable();

        let target = self.target(txn).await?;
        let pin_hpp = target.pin_hpp().to_owned();
        let platformio_ini = target.compile_platformio_ini(txn, &lib_deps).await?;

        let includes = includes.join("\n");
        let definitions = definitions.join("\n");
        let measurements = measurements.join("\n    ");
        let setups = setups.join("\n  ");
        let unauthenticated_actions = unauthenticated_actions.join("\n  ");
        let configs = configs
            .into_iter()
            .map(|c| c.1)
            .collect::<Vec<_>>()
            .join("\n");

        let main_cpp = format!(
            "#include <iop/loop.hpp>
#include <pin.hpp>
{includes}

namespace config {{
constexpr static iop::time::milliseconds measurementsInterval = 180 * 1000;
constexpr static iop::time::milliseconds unauthenticatedActionsInterval = 1000;

{device_configs}
{configs}
}}
{definitions}

auto prepareJson(iop::EventLoop & loop) noexcept -> std::unique_ptr<iop::Api::Json> {{
  IOP_TRACE();

  loop.logger().infoln(IOP_STR(\"Collect Measurements\"));
  auto json = loop.api().makeJson(IOP_FUNC, [](JsonDocument &doc) {{
    {measurements}
  }});
  iop_assert(json, IOP_STR(\"Unable to generate request payload, OOM or buffer overflow\"));
  return json;
}}

auto monitor(iop::EventLoop &loop, const iop::AuthToken &token) noexcept -> void {{
  loop.registerEvent(token, prepareJson(loop));
}}

auto unauthenticatedAct(iop::EventLoop &loop) noexcept -> void {{
  {unauthenticated_actions}
  (void) loop;
}}

namespace iop {{
auto setup(EventLoop &loop) noexcept -> void {{
  {setups}

  loop.setInterval(config::unauthenticatedActionsInterval, unauthenticatedAct);
  loop.setAuthenticatedInterval(config::measurementsInterval, monitor);
}}
}}",
        );

        let target_prototype = target.prototype(txn).await?;
        let certificate = target_prototype.latest_certificate(txn).await?;

        Compilation::new(
            txn,
            self,
            dbg!(platformio_ini),
            dbg!(main_cpp),
            dbg!(pin_hpp),
            dbg!(certificate.id()),
        )
        .await
    }

    pub async fn latest_compilation(&self, txn: &mut Transaction<'_>) -> Result<Compilation> {
        Compilation::latest_for_compiler(txn, self).await
    }

    pub async fn set_alias(
        &mut self,
        txn: &mut Transaction<'_>,
        sensor: &Sensor,
        alias: String,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE sensor_belongs_to_compiler
             SET alias = $1, updated_at = NOW()
             WHERE sensor_id = $2 AND compiler_id = $3",
        )
        .bind(alias)
        .bind(sensor.id())
        .bind(self.id())
        .execute(txn)
        .await?;
        Ok(())
    }

    pub async fn set_color(
        &mut self,
        txn: &mut Transaction<'_>,
        sensor: &Sensor,
        color: String,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE sensor_belongs_to_compiler
             SET color = $1, updated_at = NOW()
             WHERE sensor_id = $2 AND compiler_id = $3",
        )
        .bind(color)
        .bind(sensor.id())
        .bind(self.id())
        .execute(txn)
        .await?;
        Ok(())
    }

    pub async fn device_configs(&self, txn: &mut Transaction<'_>) -> Result<Vec<DeviceConfig>> {
        DeviceConfig::find_by_compiler(txn, self).await
    }

    pub async fn collection(&self, txn: &mut Transaction<'_>) -> Result<Option<Collection>> {
        Collection::find_by_compiler(txn, self).await
    }

    pub async fn organization(&self, txn: &mut Transaction<'_>) -> Result<Organization> {
        Organization::find_by_compiler(txn, self).await
    }

    pub async fn all_active(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        let comps = sqlx::query_as(
            "SELECT DISTINCT ON (compilers.id) compilers.id, compilers.target_id
             FROM compilers
             INNER JOIN collections ON collections.compiler_id = compilers.id
             INNER JOIN devices ON devices.collection_id = collections.id
             ORDER BY compilers.id, compilers.created_at DESC",
        )
        .fetch_all(txn)
        .await?;
        Ok(comps)
    }
}
