use std::env;

use diesel::pg::*;
use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

use dotenv::dotenv;

// Define here which Db will be used (postgresql, mysql, sqlite)
pub type ConnectionType = PgConnection;

pub type DbConnection = PooledConnection<ConnectionManager<ConnectionType>>;
pub type DbPool = Pool<ConnectionManager<ConnectionType>>;

lazy_static! {
    pub static ref POOL: DbPool = pool();
}

pub fn connection() -> DbConnection {
    POOL.get() 
        .expect("Did not obtain valid Diesel connection from R2D2 pool")
}

fn pool() -> DbPool {
    dotenv().ok();

    let url = match env::var("PG_DATABASE_URL") {
        Ok(val) => val,
        _ => env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment (or in .env file)")
    };

    let manager = ConnectionManager::<ConnectionType>::new(url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
