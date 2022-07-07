use crate::prelude::*;
use rand::{distributions::Alphanumeric, Rng};

#[derive(FromRow, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Migration {
    pub id: i16,
}

pub async fn run_migrations(url: &str) {
    use std::path::Path;
    use tokio::fs;

    let mut connection = sqlx::PgConnection::connect(url).await.unwrap();

    let migrations_creation_query = "CREATE TABLE IF NOT EXISTS migrations (
  id SMALLINT NOT NULL UNIQUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)";
    info!("Creating migrations table if needed");
    debug!("{}", migrations_creation_query);
    sqlx::query(migrations_creation_query)
        .execute(&mut connection)
        .await
        .expect(&format!(
            "Failed to execute query: {}",
            migrations_creation_query
        ));

    let mut transaction = sqlx::Connection::begin(&mut connection).await.unwrap();

    sqlx::query("SELECT pg_advisory_xact_lock(9128312731)")
        .execute(&mut transaction)
        .await.expect("unable to get migration lock");

    let vec: Vec<Migration> = sqlx::query_as("SELECT id FROM migrations")
        .fetch_all(&mut transaction)
        .await.expect("unable to find migrations table");
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
            .unwrap()
            .replace(".sql", "")
            .parse::<u8>()
            .unwrap_or(0);
        if (number as i16) > latest {
            files.push(number);
        }
    }
    let has_files = files.len() > 0;
    files.sort_unstable();

    // TODO: store migration query to psql row and check if they match at boot
    for file in files {
        info!("Running migration {}.sql", file);
        let path = Path::new("migrations").join(format!("{}.sql", file));
        let strings = fs::read_to_string(path).await.unwrap();
        for string in strings.split(';') {
            if string.trim().is_empty() {
                continue;
            }
            debug!("{}", string);
            sqlx::query(&format!("{};", string))
                .execute(&mut transaction)
                .await
                .expect(&format!("Failed to execute query: {}", string));
        }

        sqlx::query("INSERT INTO migrations (id) VALUES ($1)")
            .bind(file as i16)
            .execute(&mut transaction)
            .await
            .unwrap();
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
            .unwrap();
    }

    transaction.commit().await.unwrap();
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
