use crate::{logger::*, Firmware, Result, Transaction};
use derive_more::FromStr;
use serde::{Deserialize, Serialize};
use tokio::fs;

use super::compiler::{Compiler, CompilerId, CompilerView};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompilationView {
    pub id: CompilationId,
    pub compiler: CompilerView,
    pub platformio_ini: String,
    pub main_cpp: String,
    pub pin_hpp: String,
}

impl CompilationView {
    pub async fn new(txn: &mut Transaction<'_>, compilation: Compilation) -> Result<Self> {
        let compiler = compilation.compiler(txn).await?;
        Ok(Self {
            id: compilation.id(),
            platformio_ini: compilation.platformio_ini,
            main_cpp: compilation.main_cpp,
            pin_hpp: compilation.pin_hpp,
            compiler: CompilerView::new(txn, compiler).await?,
        })
    }
}

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
    pub async fn new(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
        platformio_ini: String,
        main_cpp: String,
        pin_hpp: String,
    ) -> Result<Self> {
        let id: Option<(CompilationId,)> = sqlx::query_as(
            "
            SELECT id
            FROM compilations
            WHERE compiler_id = $1
                  AND platformio_ini = $2
                  AND main_cpp = $3
                  AND pin_hpp = $4",
        )
        .bind(compiler.id())
        .bind(&platformio_ini)
        .bind(&main_cpp)
        .bind(&pin_hpp)
        .fetch_optional(&mut *txn)
        .await?;

        let mut should_compile = false;
        let id = if let Some((id,)) = id {
            id
        } else {
            should_compile = true;
            let (id,): (CompilationId,) =
                sqlx::query_as("INSERT INTO compilations (compiler_id, platformio_ini, main_cpp, pin_hpp) VALUES ($1, $2, $3, $4) RETURNING id")
                    .bind(compiler.id())
                    .bind(&platformio_ini)
                    .bind(&main_cpp)
                    .bind(&pin_hpp)
                    .fetch_one(&mut *txn)
                    .await?;
            id
        };

        let compilation = Self {
            id,
            platformio_ini,
            main_cpp,
            pin_hpp,
            compiler_id: compiler.id(),
        };

        if should_compile {
            compilation.compile(txn).await?;
        }

        Ok(compilation)
    }

    pub async fn latest_for_compiler(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
    ) -> Result<Self> {
        let comp = sqlx::query_as(
            "
            SELECT id, compiler_id, platformio_ini, main_cpp, pin_hpp
            FROM compilations
            WHERE compiler_id = $1
            ORDER BY created_at DESC",
        )
        .bind(compiler.id())
        .fetch_one(&mut *txn)
        .await?;
        Ok(comp)
    }

    pub async fn find_by_id(
        txn: &mut Transaction<'_>,
        firmware: &Firmware,
        id: CompilationId,
    ) -> Result<Self> {
        let comp = sqlx::query_as(
            "SELECT compilations.id, compiler_id, platformio_ini, main_cpp, pin_hpp
             FROM compilations
             INNER JOIN firmwares ON firmwares.compilation_id = compilations.id
             WHERE compilations.id = $1 AND firmwares.id = $2",
        )
        .bind(id)
        .bind(firmware.id())
        .fetch_one(&mut *txn)
        .await?;
        Ok(comp)
    }

    pub fn id(&self) -> CompilationId {
        self.id
    }

    pub async fn firmware(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        Firmware::find_by_compilation(txn, self).await
    }

    pub async fn compiler(&self, txn: &mut Transaction<'_>) -> Result<Compiler> {
        Compiler::find_by_compilation(txn, self).await
    }

    pub fn compiler_id(&self) -> CompilerId {
        self.compiler_id
    }

    pub async fn compile(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        // FIXME TODO: fix this, it's super dangerous, we need to run in a VM
        let compiler = self.compiler(&mut *txn).await?;
        let target = compiler.target(&mut *txn).await?;
        let prototype = target.prototype(&mut *txn).await?;
        let arch = &prototype.arch;
        let board = target.board();
        let mut env_name = vec![arch.as_str()];
        if let Some(board) = board {
            env_name.push(board);
        }
        let env_name = env_name.join("-");

        let firmware = {
            let dir = tokio::task::spawn_blocking(tempfile::tempdir).await??;
            info!("Created temp dir {dir:?}");

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

            info!("pio run -e {env_name} -d \"{}\"", dir.path().display());

            let dir_arg = dir.path().to_string_lossy();
            let mut command = tokio::process::Command::new("pio");
            command.args(["run", "-e", &env_name, "-d", &*dir_arg]);
            // TODO: stream output
            // TODO: check exit code?
            let output = command.spawn()?.wait_with_output().await?;

            if !output.stderr.is_empty() {
                error!("{}", String::from_utf8_lossy(&output.stderr));
            }
            if !output.stdout.is_empty() {
                info!("{}", String::from_utf8_lossy(&output.stdout));
            }

            // This is a big hack
            let mut filename = "firmware.bin";
            if env_name == "linux" {
                filename = "program";
            }

            println!("Read firmware");
            fs::read(
                dir.path()
                    .join(".pio")
                    .join("build")
                    .join(&env_name)
                    .join(filename),
            )
            .await?
        };

        Firmware::new(txn, self, firmware).await
    }
}
