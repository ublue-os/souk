use crate::config;
use crate::path;

use std::io;
use std::path::PathBuf;

use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;

// Read database migrations
embed_migrations!("./data/database/migrations/");

// Define 'Pool' type
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

lazy_static! {
    // Database path
    pub static ref DB_PATH: PathBuf = {
        let mut path = path::DATA.clone();
        path.push(format!("{}.db",config::NAME));
        path
    };

    // Database R2D2 connection pool
    static ref POOL: Pool = init_connection_pool(DB_PATH.to_str().unwrap());
}

// Returns a R2D2 SqliteConnection
pub fn get_connection() -> Pool {
    POOL.clone()
}

// Inits database connection pool, and run migrations.
// If there's no database, it get's created automatically.
fn init_connection_pool(db_path: &str) -> Pool {
    let manager = ConnectionManager::<SqliteConnection>::new(db_path);
    let pool = r2d2::Pool::builder().max_size(1).build(manager).expect("Failed to create pool.");

    let db = pool.get().expect("Failed to initialize pool.");
    run_migrations(&*db).expect("Failed to run migrations during init.");

    info!("Initialized database connection pool.");
    pool
}

fn run_migrations(connection: &SqliteConnection) -> Result<(), diesel::migration::RunMigrationsError> {
    info!("Running DB Migrations...");
    embedded_migrations::run_with_output(connection, &mut io::stdout()).map_err(From::from)
}
