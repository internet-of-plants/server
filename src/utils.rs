use std::path::PathBuf;

use crate::{logger::*, Result};
use rand::{distributions::Alphanumeric, Rng};
use sqlx::Connection;

#[derive(sqlx::FromRow, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Migration {
    pub id: i16,
    pub code: String,
}

pub async fn run_migrations(url: &str) {
    use std::path::Path;
    use tokio::fs;

    let mut connection = sqlx::PgConnection::connect(url)
        .await
        .expect("unable to connect to pg");

    let migrations_creation_query = "CREATE TABLE IF NOT EXISTS migrations (
  id         SMALLINT    NOT NULL UNIQUE,
  code       TEXT        NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)";
    info!("Creating migrations table if needed");
    debug!("{}", migrations_creation_query);
    sqlx::query(migrations_creation_query)
        .execute(&mut connection)
        .await
        .unwrap_or_else(|_| panic!("Failed to execute query: {}", migrations_creation_query));

    let mut transaction = sqlx::Connection::begin(&mut connection)
        .await
        .expect("unable to connect to pg");

    sqlx::query("SELECT pg_advisory_xact_lock(9128312731)")
        .execute(&mut transaction)
        .await
        .expect("unable to get migration lock");

    let vec: Vec<Migration> = sqlx::query_as("SELECT id, code FROM migrations")
        .fetch_all(&mut transaction)
        .await
        .expect("unable to find migrations table");
    let latest = vec.iter().max().map_or(0, |m| m.id);

    let mut files = Vec::new();
    let mut reader = fs::read_dir("migrations")
        .await
        .expect("Unable to find migrations folder");
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
            let obj = vec.get((number - 1) as usize);
            if obj.is_some()
                && obj
                    != Some(&Migration {
                        id: number,
                        code: code.clone(),
                    })
            {
                error!("Migration found: {:#?}", vec.get((number - 1) as usize));
                error!("Migration expected: {:#?}", Migration { id: number, code });
                panic!("Migration {} changed", number);
            }
        }
    }
    let has_files = !files.is_empty();
    files.sort_unstable();

    for file in files {
        info!("Running migration {}.sql", file);
        let path = Path::new("migrations").join(format!("{}.sql", file));
        let code = fs::read_to_string(path)
            .await
            .expect("unable to open migration file");
        for string in code.split(';') {
            if string.trim().is_empty() {
                continue;
            }
            debug!("{}", string);
            sqlx::query(&format!("{};", string))
                .execute(&mut transaction)
                .await
                .unwrap_or_else(|_| panic!("Failed to execute query: {}", string));
        }

        sqlx::query("INSERT INTO migrations (id, code) VALUES ($1, $2)")
            .bind(file as i16)
            .bind(&code)
            .execute(&mut transaction)
            .await
            .expect("unable to insert new migration");
        info!(
            "Has migrations: {:?}",
            sqlx::query_as::<_, (i16,)>("SELECT id FROM migrations ORDER BY id ASC")
                .fetch_all(&mut transaction)
                .await
        );
    }

    if has_files {
        crate::db::sensor_prototype::builtin::create_builtin(&mut transaction)
            .await
            .expect("failed to create builtins");
    }

    transaction
        .commit()
        .await
        .expect("transaction commit failed");
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
