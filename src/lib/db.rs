use diesel::PgConnection;
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
    #[cfg(not(release))]
    dotenv().ok();

    let url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment (or in .env file)");
    let manager = ConnectionManager::<ConnectionType>::new(url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

/// Create new connection pool to test db
#[cfg(test)]
pub fn test_pool() -> DbPool {
    use diesel::Connection;

    dotenv().ok();

    let url = env::var("DATABASE_TEST_URL")
        .expect("DATABASE_TEST_URL must be set in environment (or in .env file)");
    let manager = ConnectionManager::<ConnectionType>::new(url);
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to create pool.");
    (&*connection(&pool).unwrap())
        .begin_test_transaction()
        .unwrap();
    pool
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Assure pool creation and connection retrieval is correct
    fn db() {
        &*connection(&pool()).unwrap();
        &*connection(&test_pool()).unwrap();
    }
}
