use crate::{
    logger::*, CertificateId, Compiler, CompilerId, Error, Firmware, Result, SensorId, Transaction,
};
use derive::id;
use derive_get::Getters;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Getters, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CompilationView {
    #[copy]
    id: CompilationId,
    platformio_ini: String,
    main_cpp: String,
    pin_hpp: String,
}

impl CompilationView {
    pub fn new(compilation: Compilation) -> Self {
        Self {
            id: compilation.id(),
            platformio_ini: compilation.platformio_ini,
            main_cpp: compilation.main_cpp,
            pin_hpp: compilation.pin_hpp,
        }
    }
}

#[id]
pub struct CompilationId;

#[derive(sqlx::FromRow, Getters, Debug)]
pub struct Compilation {
    #[copy]
    id: CompilationId,
    #[copy]
    compiler_id: CompilerId,
    platformio_ini: String,
    main_cpp: String,
    pin_hpp: String,
    #[copy]
    certificate_id: CertificateId,
}

impl Compilation {
    pub async fn new(
        txn: &mut Transaction<'_>,
        compiler: &Compiler,
        platformio_ini: String,
        main_cpp: String,
        pin_hpp: String,
        certificate_id: CertificateId,
    ) -> Result<Self> {
        let id: Option<(CompilationId,)> = sqlx::query_as(
            "
            SELECT id
            FROM compilations
            WHERE compiler_id = $1
                  AND platformio_ini = $2
                  AND main_cpp = $3
                  AND pin_hpp = $4
                  AND certificate_id = $5",
        )
        .bind(compiler.id())
        .bind(&platformio_ini)
        .bind(&main_cpp)
        .bind(&pin_hpp)
        .bind(certificate_id)
        .fetch_optional(&mut *txn)
        .await?;

        let mut should_compile = false;
        let id = if let Some((id,)) = id {
            id
        } else {
            should_compile = true;
            let (id,): (CompilationId,) =
                sqlx::query_as("INSERT INTO compilations (compiler_id, platformio_ini, main_cpp, pin_hpp, certificate_id) VALUES ($1, $2, $3, $4, $5) RETURNING id")
                    .bind(compiler.id())
                    .bind(&platformio_ini)
                    .bind(&main_cpp)
                    .bind(&pin_hpp)
                    .bind(certificate_id)
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
            certificate_id,
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
            "SELECT id, compiler_id, platformio_ini, main_cpp, pin_hpp, certificate_id
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
            "SELECT compilations.id, compiler_id, platformio_ini, main_cpp, pin_hpp, certificate_id
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

    pub async fn all_active(txn: &mut Transaction<'_>) -> Result<Vec<Self>> {
        let comps = sqlx::query_as(
            "SELECT DISTINCT ON (compilations.compiler_id) compilations.compiler_id, compilations.id, platformio_ini, main_cpp, pin_hpp, certificate_id
             FROM compilations
             INNER JOIN collections ON collections.compiler_id = compilations.compiler_id
             INNER JOIN devices ON devices.collection_id = collections.id
             ORDER BY compilations.compiler_id, compilations.created_at DESC",
        )
        .fetch_all(&mut *txn)
        .await?;
        Ok(comps)
    }

    pub async fn firmware(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        Firmware::latest_by_compilation(txn, self).await
    }

    pub async fn compiler(&self, txn: &mut Transaction<'_>) -> Result<Compiler> {
        Compiler::find_by_compilation(txn, self).await
    }

    pub async fn is_outdated(&self, txn: &mut Transaction<'_>) -> Result<bool> {
        let compiler = self.compiler(&mut *txn).await?;
        let target = compiler.target(txn).await?;
        let target_prototype = target.prototype(txn).await?;

        let dependencies = sqlx::query_as::<_, (String, Option<SensorId>, String)>(
            "SELECT repo_url, sensor_id, commit_hash
             FROM dependency_belongs_to_compilation
             WHERE compilation_id = $1",
        )
        .bind(self.id)
        .fetch_all(&mut *txn)
        .await?;

        // Is there any RCE danger in cloning a git repo?
        for sensor in compiler.sensors(txn).await? {
            let sid = sensor.id();

            for dependency in sensor.prototype().dependencies() {
                let url = dependency.repo_url().clone();
                let branch = dependency.branch().clone();
                let commit_hash = tokio::task::spawn_blocking(move || {
                    let dir = tempfile::tempdir()?;
                    let repo = git2::build::RepoBuilder::new()
                        .branch(&branch)
                        .clone(&url, dir.path())?;

                    let object = repo.revparse_single(&branch)?;
                    let commit = object.peel_to_commit()?;
                    let commit_hash = commit.id().to_string();
                    Ok::<_, Error>(commit_hash)
                })
                .await??;

                if dependencies
                    .iter()
                    .any(|(url, sensor_id, expected_commit_hash)| {
                        url == dependency.repo_url()
                            && *sensor_id == Some(sid)
                            && &commit_hash != expected_commit_hash
                    })
                {
                    return Ok(true);
                }
            }
        }

        for dependency in target_prototype.dependencies(txn).await? {
            let url = dependency.repo_url().clone();
            let branch = dependency.branch().clone();
            let commit_hash = tokio::task::spawn_blocking(move || {
                let dir = tempfile::tempdir()?;
                let repo = git2::build::RepoBuilder::new()
                    .branch(&branch)
                    .clone(&url, dir.path())?;

                let object = repo.revparse_single(&branch)?;
                let commit = object.peel_to_commit()?;
                let commit_hash = commit.id().to_string();
                Ok::<_, Error>(commit_hash)
            })
            .await??;

            if dependencies
                .iter()
                .any(move |(dep_url, sensor_id, expected_commit_hash)| {
                    dep_url == dependency.repo_url()
                        && sensor_id.is_none()
                        && &commit_hash != expected_commit_hash
                })
            {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn compile_if_outdated(&self, txn: &mut Transaction<'_>) -> Result<()> {
        if self.is_outdated(txn).await? {
            self.compile(txn).await?;
        }
        Ok(())
    }

    pub async fn compile(&self, txn: &mut Transaction<'_>) -> Result<Firmware> {
        info!("Compiling: {:?}", self.id);
        // FIXME TODO: fix this, it's super dangerous, we need to run in a VM
        let compiler = self.compiler(txn).await?;
        let target = compiler.target(txn).await?;
        let prototype = target.prototype(txn).await?;
        let arch = prototype.arch();
        let board = target.board();
        let mut env_name = vec![arch.as_str()];
        if let Some(board) = board {
            env_name.push(board);
        }
        let env_name = env_name.join("-");

        for sensor in compiler.sensors(txn).await? {
            for dependency in sensor.prototype().dependencies() {
                let url = dependency.repo_url().clone();
                let branch = dependency.branch().clone();
                // Is there any RCE danger in cloning a git repo?
                let commit_hash = tokio::task::spawn_blocking(move || {
                    let dir = tempfile::tempdir()?;
                    let repo = git2::build::RepoBuilder::new()
                        .branch(&branch)
                        .clone(&url, dir.path())?;

                    let object = repo.revparse_single(&branch)?;
                    let commit = object.peel_to_commit()?;
                    let commit_hash = commit.id().to_string();
                    Ok::<_, Error>(commit_hash)
                })
                .await??;

                sqlx::query("INSERT INTO dependency_belongs_to_compilation (repo_url, branch, sensor_id, commit_hash, compilation_id) VALUES ($1, $2, $3, $4, $5)
                             ON CONFLICT (repo_url, compilation_id) DO UPDATE SET commit_hash = $4")
                    .bind(dependency.repo_url())
                    .bind(dependency.branch())
                    .bind(sensor.id())
                    .bind(&commit_hash)
                    .bind(self.id())
                    .execute(&mut *txn)
                    .await?;
            }
        }

        for dependency in prototype.dependencies(txn).await? {
            let url = dependency.repo_url().clone();
            let branch = dependency.branch().clone();
            let commit_hash = tokio::task::spawn_blocking(move || {
                let dir = tempfile::tempdir()?;
                let repo = git2::build::RepoBuilder::new()
                    .branch(&branch)
                    .clone(&url, dir.path())?;

                let object = repo.revparse_single(&branch)?;
                let commit = object.peel_to_commit()?;
                let commit_hash = commit.id().to_string();
                Ok::<_, Error>(commit_hash)
            })
            .await??;

            sqlx::query(
                "INSERT INTO dependency_belongs_to_compilation (repo_url, branch, commit_hash, compilation_id)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT (repo_url, compilation_id) DO UPDATE SET commit_hash = $3",
            )
            .bind(dependency.repo_url())
            .bind(dependency.branch())
            .bind(&commit_hash)
            .bind(self.id())
            .execute(&mut *txn)
            .await?;
        }

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
