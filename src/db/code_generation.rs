use crate::db::firmware::Firmware;
use crate::db::sensor::*;
use crate::db::target::*;
use crate::prelude::*;
use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Serialize, Deserialize, sqlx::Type, Clone, Copy, Debug, PartialEq, Eq, FromStr)]
#[sqlx(transparent)]
pub struct CompilationId(pub i64);

impl CompilationId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Compilation {
    id: CompilationId,
    compiler_id: CompilerId,
    pub platformio_ini: String,
    pub main_cpp: String,
    pub pin_hpp: String,
}

impl Compilation {
    pub fn id(&self) -> CompilationId {
        self.id
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        compiler_id: CompilerId,
        platformio_ini: String,
        main_cpp: String,
        pin_hpp: String,
    ) -> Result<Self> {
        let (id,): (CompilationId,) =
            sqlx::query_as("INSERT INTO compilations (compiler_id, platformio_ini, main_cpp, pin_hpp) VALUES ($1, $2, $3, $4) RETURNING id")
                .bind(compiler_id)
                .bind(&platformio_ini)
                .bind(&main_cpp)
                .bind(&pin_hpp)
                .fetch_one(&mut *txn)
                .await?;
        Ok(Self {
            id,
            platformio_ini,
            main_cpp,
            pin_hpp,
            compiler_id,
        })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: CompilationId) -> Result<Self> {
        let comp = sqlx::query_as("SELECT id, compiler_id, platformio_ini, main_cpp, pin_hpp FROM compilations WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *txn)
            .await?;
        Ok(comp)
    }

    pub async fn list(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        let comps = sqlx::query_as(
            "SELECT id, compiler_id, platformio_ini, main_cpp, pin_hpp FROM compilations",
        )
        .fetch_all(&mut *txn)
        .await?;
        Ok(comps)
    }

    pub async fn compiler(&self, txn: &mut Transaction<'_>) -> Result<Compiler> {
        Compiler::find_by_id(txn, self.compiler_id).await
    }

    pub async fn compile(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        // FIXME TODO: fix this, it's super dangerous, we need to run in a VM
        let compiler = self.compiler(&mut *txn).await?;
        let target = compiler.target(&mut *txn).await?;
        let prototype = target.prototype(&mut *txn).await?;
        let arch = &prototype.arch;
        let board = &target.board(&mut *txn).await?.board;
        let build_name = format!("{arch}-{board}");

        let firmware = {
            let dir = tokio::task::spawn_blocking(|| tempfile::tempdir()).await??;
            fs::write(
                dir.path().join("platformio.ini"),
                self.platformio_ini.as_bytes(),
            )
            .await?;
            fs::create_dir(dir.path().join("src")).await?;
            fs::write(
                dir.path().join("src").join("main.cpp"),
                self.main_cpp.as_bytes(),
            )
            .await?;
            fs::create_dir(dir.path().join("include")).await?;
            fs::write(
                dir.path().join("include").join("pin.hpp"),
                self.pin_hpp.as_bytes(),
            )
            .await?;

            info!("pio run -e {build_name} -d \"{}\"", dir.path().display());

            // TODO: is dir.path().display() the correct approach?
            let dir_arg = dir.path().display().to_string();
            let mut command = tokio::process::Command::new("pio");
            command.args(["run", "-e", &build_name, "-d", dir_arg.as_str()]);
            // TODO: stream output
            // TODO: check exit code?
            let output = command.spawn()?.wait_with_output().await?;

            if !output.stderr.is_empty() {
                error!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.stdout.is_empty() {
                info!("{}", String::from_utf8_lossy(&output.stdout));
            }

            fs::read(
                dir.path()
                    .join(".pio")
                    .join("build")
                    .join(&build_name)
                    .join("firmware.bin"),
            )
            .await?
        };

        Firmware::new(txn, Some(self.id), firmware).await
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
    pub fn id(&self) -> CompilerId {
        self.id
    }

    pub async fn new(
        txn: &mut Transaction<'_>,
        target_id: TargetId,
        sensor_ids: Vec<SensorId>,
    ) -> Result<Self> {
        let (id,): (CompilerId,) =
            sqlx::query_as("INSERT INTO compilers (target_id) VALUES ($1) RETURNING id")
                .bind(target_id)
                .fetch_one(&mut *txn)
                .await?;
        for sensor_id in sensor_ids {
            sqlx::query(
                "INSERT INTO sensor_belongs_to_compiler (sensor_id, compiler_id) VALUES ($1, $2)",
            )
            .bind(&sensor_id)
            .bind(&id)
            .execute(&mut *txn)
            .await?;
        }
        Ok(Self { id, target_id })
    }

    pub async fn find_by_id(txn: &mut Transaction<'_>, id: CompilerId) -> Result<Self> {
        let comp = sqlx::query_as("SELECT id, target_id FROM compilers WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *txn)
            .await?;
        Ok(comp)
    }

    pub async fn target(&self, txn: &mut Transaction<'_>) -> Result<Target> {
        Target::find_by_id(txn, self.target_id).await
    }

    pub async fn sensors(&self, txn: &mut Transaction<'_>) -> Result<Vec<Sensor>> {
        let sensor_ids: Vec<(SensorId,)> = sqlx::query_as(
            "SELECT sensor_id FROM sensor_belongs_to_compiler WHERE compiler_id = $1",
        )
        .bind(&self.id)
        .fetch_all(&mut *txn)
        .await?;
        let mut sensors = Vec::with_capacity(sensor_ids.len());
        for (sensor_id,) in sensor_ids {
            sensors.push(Sensor::find_by_id(&mut *txn, sensor_id).await?);
        }
        Ok(sensors)
    }

    pub async fn compile(&self, txn: &mut Transaction<'_>) -> Result<Compilation> {
        let sensors = self.sensors(&mut *txn).await?;

        let mut lib_deps = Vec::new();
        let mut includes = Vec::new();
        let mut definitions = Vec::new();
        let mut measurements = Vec::new();
        let mut setups = Vec::new();
        let mut configs = Vec::with_capacity(sensors.len());
        for sensor in &sensors {
            let prototype = sensor.prototype(&mut *txn).await?;
            lib_deps.extend(prototype.dependencies(&mut *txn).await?);
            includes.extend(
                prototype
                    .includes(&mut *txn)
                    .await?
                    .into_iter()
                    .map(|name| format!("#include <{name}>")),
            );
            definitions.extend(prototype.definitions(&mut *txn).await?);
            measurements.extend(
                prototype
                    .measurements(&mut *txn)
                    .await?
                    .into_iter()
                    .map(|m| format!("doc[\"{}\"] = {}", m.name, m.value)),
            );
            setups.extend(prototype.setups(&mut *txn).await?);

            for c in sensor.configs(&mut *txn).await? {
                let req = c.request(&mut *txn).await?;
                configs.push(format!(
                    "constexpr static {} {} = {};",
                    req.ty(&mut *txn).await?.name,
                    req.name,
                    c.value
                ));
            }
        }

        let target = self.target(&mut *txn).await?;
        let pin_hpp = target.board(&mut *txn).await?.pin_hpp().to_owned();
        let platformio_ini = target.compile_platformio_ini(&mut *txn, &lib_deps).await?;

        let includes = includes.join("\n");
        let definitions = definitions.join("\n");
        let measurements = measurements.join("\n    ");
        let setups = setups.join("\n  ");
        let configs = configs.join("\n");

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

  const auto json = loop.api().makeJson(IOP_FUNC, [](JsonDocument &doc) {{
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
}}
",
        );
        Ok(Compilation::new(&mut *txn, self.id, platformio_ini, main_cpp, pin_hpp).await?)
    }
}
