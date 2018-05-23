use diesel::pg::*;
use dotenv::dotenv;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;
use std::env;

use lib::error::Error;

pub type ConnectionType = PgConnection;

pub type DbConnection = PooledConnection<ConnectionManager<ConnectionType>>;
pub type DbPool = Pool<ConnectionManager<ConnectionType>>;

/// Obtain connection from pool
pub fn connection(pool: &DbPool) -> Result<DbConnection, Error> {
    Ok(pool.get()?)
}

/// Create new connection pool
pub fn pool() -> DbPool {
    dotenv().ok();

    #[cfg(not(test))]
    let url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment (or in .env file)");

    #[cfg(test)]
    let url = env::var("DATABASE_TEST_URL")
        .expect("DATABASE_TEST_URL must be set in environment (or in .env file)");

    let manager = ConnectionManager::<ConnectionType>::new(url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assure pool creation and connection retrieval is correct
    fn db() {
        &*connection(&pool()).unwrap();
    }
}
