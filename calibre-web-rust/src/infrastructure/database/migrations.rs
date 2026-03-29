// src/infrastructure/database/migrations.rs
use sqlx::{PgPool, Executor};
use std::path::Path;

pub async fn run_migrations(pool: &PgPool, migrations_dir: &Path) -> Result<(), sqlx::Error> {
    let mut conn = pool.begin().await?;

    // Read and execute migration files in order
    let migration_files = std::fs::read_dir(migrations_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map(|ext| ext == "sql").unwrap_or(false))
        .filter(|entry| entry.file_name().to_string_lossy().ends_with(".up.sql"))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    for migration_file in migration_files {
        let migration_sql = std::fs::read_to_string(&migration_file)?;
        conn.execute(&*migration_sql).await?;
        tracing::info!("Executed migration: {:?}", migration_file.file_name());
    }

    conn.commit().await?;
    Ok(())
}
