//! Thread-safe player database state and initialization.
//!
//! Provides the connection pool used across the whole application and runs
//! pending Diesel migrations on startup. Seed data is applied via migrations
//! rather than Rust code, so a fresh database is fully populated on first run
//! without any additional setup.

use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// r2d2 connection pool backed by SQLite via Diesel.
///
/// A type alias for `Pool<ConnectionManager<SqliteConnection>>`. Axum shares
/// this across async worker threads via router state. The pool manages connections internally
/// and hands them out on demand — callers call `.get()` to borrow a
/// `PooledConnection`, which is returned to the pool automatically on drop.
pub type PlayerCollection = Pool<ConnectionManager<SqliteConnection>>;

/// Opens or creates the SQLite database, runs any pending migrations
/// (schema + seed data), and returns a connection pool.
///
/// The database path is read from the `STORAGE_PATH` environment variable,
/// defaulting to `storage/players-sqlite3.db`. Migrations are idempotent:
/// Diesel tracks applied migrations in `__diesel_schema_migrations` and skips
/// anything already recorded, so existing data is never overwritten.
pub fn initialize_database() -> PlayerCollection {
    let path = std::env::var("STORAGE_PATH")
        .ok()
        .and_then(|v| {
            let t = v.trim().to_string();
            if t.is_empty() { None } else { Some(t) }
        })
        .unwrap_or_else(|| "storage/players-sqlite3.db".to_string());
    if let Some(parent) = std::path::Path::new(&path).parent() {
        std::fs::create_dir_all(parent).expect("Failed to create storage directory");
    }
    let manager = ConnectionManager::<SqliteConnection>::new(&path);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to build connection pool");
    pool.get()
        .expect("Failed to get connection from pool")
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");
    pool
}

/// Opens an in-memory SQLite database, runs all migrations (schema + seed).
///
/// `max_size(1)` ensures a single shared connection for the lifetime of the
/// pool, which is required for in-memory SQLite: each connection would
/// otherwise get its own isolated database.
///
/// Used exclusively in tests.
#[allow(dead_code)]
pub fn initialize_test_database() -> PlayerCollection {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to build test connection pool");
    pool.get()
        .expect("Failed to get test connection from pool")
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run test migrations");
    pool
}

/// Opens an in-memory SQLite database and runs only the schema migration
/// (the first migration), leaving the `players` table empty.
///
/// Used exclusively in tests that need an empty but structurally valid database.
#[allow(dead_code)]
pub fn initialize_empty_test_database() -> PlayerCollection {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to build empty test connection pool");
    pool.get()
        .expect("Failed to get empty test connection from pool")
        .run_next_migration(MIGRATIONS)
        .expect("Failed to run schema migration");
    pool
}
