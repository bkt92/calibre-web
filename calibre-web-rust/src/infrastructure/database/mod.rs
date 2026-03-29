// src/infrastructure/database/mod.rs
pub mod postgres;
pub mod migrations;

pub use postgres::{create_postgres_pool};

pub type PgPool = sqlx::PgPool;

// Helper to run migrations on startup
pub async fn ensure_migrations_run(pool: &PgPool) -> Result<(), sqlx::Error> {
    let migrations_dir = std::path::Path::new("migrations");
    if migrations_dir.exists() {
        migrations::run_migrations(pool, migrations_dir).await?;
    }
    Ok(())
}
