use std::{fmt::Write, path::Path, path::PathBuf};

use crate::{logger::*, Pool, Result};
use derive_get::Getters;
use rand::{distributions::Alphanumeric, Rng};
use tokio::fs;

#[derive(sqlx::FromRow, Getters, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Migration {
    #[copy]
    id: i16,
    code_hash: String,
}

fn hash(msg: &str) -> Result<String> {
    let md5 = md5::compute(msg);
    let md5 = &*md5;
    let mut hash = String::with_capacity(md5.len() * 2);
    for byte in md5 {
        write!(hash, "{:02X}", byte)?;
    }
    Ok(hash)
}

pub async fn run_migrations(pool: &'static Pool) {
    let mut txn = pool.begin().await.expect("unable to start transaction");

    let migrations_creation_query = "CREATE TABLE IF NOT EXISTS migrations (
  id         SMALLINT    NOT NULL UNIQUE,
  code_hash  TEXT        NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)";
    info!("Creating migrations table if needed");
    debug!("{}", migrations_creation_query);
    sqlx::query(migrations_creation_query)
        .execute(&mut txn)
        .await
        .unwrap_or_else(|err| {
            panic!(
                "Failed to execute query: {} ({})",
                migrations_creation_query, err
            )
        });
    txn.commit().await.expect("unable to commit transaction");

    let mut txn = pool.begin().await.expect("unable to start transaction");

    sqlx::query("SELECT pg_advisory_xact_lock(9128312731)")
        .execute(&mut txn)
        .await
        .expect("unable to get migration lock");

    let vec: Vec<Migration> = sqlx::query_as("SELECT id, code_hash FROM migrations")
        .fetch_all(&mut txn)
        .await
        .expect("unable to find migrations table");
    let latest = vec.iter().max().map_or(0, |m| m.id);

    txn.commit().await.expect("unable to commit transaction");

    let mut files = Vec::new();
    let mut reader = fs::read_dir("migrations")
        .await
        .expect("Unable to find migrations folder");

    // TODO: this is bugged, doesnt ensure file sorting mode
    while let Some(entry) = reader
        .next_entry()
        .await
        .expect("Failed to read migration file")
    {
        let number = entry
            .file_name()
            .to_str()
            .expect("migration filename was not utf8")
            .replace(".sql", "")
            .parse::<i16>()
            .expect("migration filename was invalid, must be a number and they must not repeat");
        if number < 1 {
            panic!("Migration number must be >=1: {}", number);
        } else if number > latest {
            if files.contains(&number) {
                panic!("Migration {} is duplicated", number);
            }
            files.push(number);
        } else {
            let path = PathBuf::from("migrations").join(entry.file_name());
            info!("Reading migration file: {:?}", path);
            let code = fs::read_to_string(path)
                .await
                .expect("unable to read migration file");
            let code_hash = hash(&code).expect("unable to hash code");
            let obj = vec.get((number - 1) as usize);
            if obj.is_some()
                && obj
                    != Some(&Migration {
                        id: number,
                        code_hash: code_hash.clone(),
                    })
            {
                error!("Migration found: {:#?}", vec.get((number - 1) as usize));
                error!(
                    "Migration expected: {:#?}",
                    Migration {
                        id: number,
                        code_hash
                    }
                );
                panic!("Migration {} changed", number);
            }
        }
    }
    let has_files = !files.is_empty();
    files.sort_unstable();

    for file in files {
        info!("Running migration {}.sql", file);
        let mut txn = pool.begin().await.expect("unable to start transaction");

        let path = Path::new("migrations").join(format!("{}.sql", file));
        let code = fs::read_to_string(path)
            .await
            .expect("unable to open migration file");
        let code_hash = hash(&code).expect("unable to hash code");

        for string in code.split(';') {
            if string.trim().is_empty() {
                continue;
            }
            debug!("{}", string);
            sqlx::query(&format!("{};", string))
                .execute(&mut txn)
                .await
                .unwrap_or_else(|_| panic!("Failed to execute query: {}", string));
        }

        sqlx::query("INSERT INTO migrations (id, code_hash) VALUES ($1, $2)")
            .bind(file)
            .bind(&code_hash)
            .execute(&mut txn)
            .await
            .expect("unable to insert new migration");
        info!(
            "Has migrations: {:?}",
            sqlx::query_as::<_, (i16,)>("SELECT id FROM migrations ORDER BY id ASC")
                .fetch_all(&mut txn)
                .await
        );
        txn.commit().await.expect("unable to commit transaction");
    }

    if has_files {
        let mut txn = pool.begin().await.expect("unable to start transaction");
        crate::db::builtin::create_builtin(&mut txn)
            .await
            .expect("failed to create builtins");
        txn.commit().await.expect("unable to commit transaction");
    }
}

pub fn random_string(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

pub fn hash_password(password: &str) -> Result<String> {
    Ok(bcrypt::hash(password, bcrypt::DEFAULT_COST)?)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    Ok(bcrypt::verify(password, hash)?)
}
