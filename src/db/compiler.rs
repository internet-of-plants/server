use crate::db::sensor::*;
use crate::db::target::*;
use crate::prelude::*;
use derive_more::FromStr;
use handlebars::Handlebars;
use random_color::RandomColor;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::compilation::Compilation;
use super::device::Device;
use super::firmware::FirmwareView;
use super::sensor::config_request::ConfigRequest;
use super::sensor::SensorView;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompilerView {
    pub id: CompilerId,
    pub sensors: Vec<SensorView>,
    pub target: TargetView,
    pub latest_firmware: FirmwareView,
}

impl CompilerView {
    pub async fn new(
        txn: &mut Transaction<'_>,
        compiler: Compiler,
        device: &Device,
    ) -> Result<Self> {
        let sensors = compiler.sensors(txn, device).await?;

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
        })
    }
}

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct CompilerId(i64);

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
        sensors_and_alias: &[(Sensor, String)],
        device: &Device,
    ) -> Result<(Self, Compilation)> {
        let id: Option<(CompilerId,)> = sqlx::query_as(
            "
            SELECT compiler_id
            FROM (SELECT COUNT(sensor_id) as count, compiler_id
                  FROM (SELECT bt.sensor_id, bt.compiler_id
                        FROM sensor_belongs_to_compiler as bt
                        WHERE bt.sensor_id = ANY($1)) as s_bt_c
                  GROUP BY compiler_id) as sensors_count
            INNER JOIN compilers on compilers.id = compiler_id
            WHERE count = $2 AND target_id = $3",
        )
        .bind(
            &sensors_and_alias
                .iter()
                .map(|s| s.0.id.0)
                .collect::<Vec<_>>(),
        )
        .bind(&(sensors_and_alias.len() as i64))
        .bind(target.id())
        .fetch_optional(&mut *txn)
        .await?;

        let mut should_compile = false;
        let id = if let Some((id,)) = id {
            id
        } else {
            should_compile = true;
            let (id,): (CompilerId,) =
                sqlx::query_as("INSERT INTO compilers (target_id) VALUES ($1) RETURNING id")
                    .bind(target.id())
                    .fetch_one(&mut *txn)
                    .await?;
            id
        };
        for (sensor, alias) in sensors_and_alias {
            let color = RandomColor::new().to_hsl_string();
            sqlx::query(
                    "INSERT INTO sensor_belongs_to_compiler (sensor_id, compiler_id, alias, color, device_id) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
                )
                    .bind(sensor.id())
                    .bind(&id)
                    .bind(&alias)
                    .bind(&color)
                    .bind(device.id())
                    .execute(&mut *txn)
                    .await?;
        }
        let compiler = Self {
            id,
            target_id: target.id(),
        };
        let compilation = if should_compile {
            compiler.compile(&mut *txn, device).await?
        } else {
            compiler.latest_compilation(&mut *txn).await?
        };
        Ok((compiler, compilation))
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: CompilerId) -> Result<Self> {
        let comp = sqlx::query_as("SELECT id, target_id FROM compilers WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *txn)
            .await?;
        Ok(comp)
    }

    pub fn id(&self) -> CompilerId {
        self.id
    }

    pub async fn target(&self, txn: &mut Transaction<'_>) -> Result<Target> {
        Target::find_by_id(txn, self.target_id).await
    }

    pub async fn sensors(
        &self,
        txn: &mut Transaction<'_>,
        device: &Device,
    ) -> Result<Vec<SensorView>> {
        SensorView::list_for_compiler(txn, self, device).await
    }

    pub async fn compile(&self, txn: &mut Transaction<'_>, device: &Device) -> Result<Compilation> {
        let sensors = self.sensors(&mut *txn, device).await?;

        let mut lib_deps = Vec::new();
        let mut includes = Vec::new();
        let mut definitions = Vec::new();
        let mut measurements = Vec::new();
        let mut setups = Vec::new();
        let mut configs = Vec::with_capacity(sensors.len());
        for (index, sensor) in sensors.iter().enumerate() {
            let prototype = &sensor.prototype;
            lib_deps.extend(prototype.dependencies.clone());
            includes.extend(
                prototype
                    .includes
                    .iter()
                    .map(|name| format!("#include <{name}>")),
            );
            definitions.push(
                prototype
                    .definitions
                    .iter()
                    .map(|definition| {
                        let reg = Handlebars::new();
                        reg.render_template(&definition, &json!({ "index": index }))
                    })
                    .collect::<Result<Vec<String>, _>>()?
                    .join("\n"),
            );
            measurements.push(
                prototype
                    .measurements
                    .iter()
                    .map(|m| {
                        let reg = Handlebars::new();
                        let name = reg.render_template(&m.name, &json!({ "index": index }))?;
                        let value = reg.render_template(&m.value, &json!({ "index": index }))?;
                        Ok(format!("doc[\"{}\"] = {}", name, value))
                    })
                    .collect::<Result<Vec<String>, Error>>()?
                    .join("\n    "),
            );
            setups.extend(
                prototype
                    .setups
                    .iter()
                    .map(|setup| {
                        let reg = Handlebars::new();
                        reg.render_template(&setup, &json!({ "index": index }))
                    })
                    .collect::<Result<Vec<String>, _>>()?,
            );

            for c in &sensor.configurations {
                let req = ConfigRequest::find_by_id(&mut *txn, c.request_id).await?;
                let reg = Handlebars::new();
                let name = reg.render_template(&req.name, &json!({ "index": index }))?;
                configs.push((
                    req.name.clone(),
                    format!(
                        "constexpr static {} {} = {};",
                        req.ty(&mut *txn).await?.name,
                        name,
                        c.value
                    ),
                ));
            }
        }
        lib_deps.dedup();
        lib_deps.sort_unstable();

        includes.dedup();
        includes.sort_unstable();

        measurements.sort_unstable();

        configs.sort_by(|a, b| a.0.cmp(&b.0));

        setups.sort_unstable();

        definitions.sort_unstable();

        let target = self.target(&mut *txn).await?;
        let pin_hpp = target.pin_hpp().to_owned();
        let platformio_ini = target.compile_platformio_ini(&mut *txn, &lib_deps).await?;

        let includes = includes.join("\n");
        let definitions = definitions.join("\n");
        let measurements = measurements.join("\n    ");
        let setups = setups.join("\n  ");
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
{configs}
}}
{definitions}

auto reportMeasurements(iop::EventLoop &loop, const iop::AuthToken &token) noexcept -> void {{
  loop.logger().debug(IOP_STR(\"Handle Measurements\"));

  const auto json = loop.api().makeJson(IOP_FUNC, [](JsonDocument &doc) {{\
    {measurements}
  }});
  if (!json) iop_panic(IOP_STR(\"Unable to send measurements, buffer overflow\"));

  loop.registerEvent(token, *json);
}}

namespace iop {{
auto setup(EventLoop &loop) noexcept -> void {{
  {setups}
  loop.setAuthenticatedInterval(config::measurementsInterval, reportMeasurements);
}}
}}",
        );
        Ok(Compilation::new(&mut *txn, self, platformio_ini, main_cpp, pin_hpp).await?)
    }

    pub async fn latest_compilation(&self, txn: &mut Transaction<'_>) -> Result<Compilation> {
        Compilation::latest_for_compiler(txn, self.id).await
    }
}
