use sqlx::PgPool;
use tracing::info;

pub async fn run_migrations(pool: &PgPool) -> crate::AppResult<()> {
    info!("Running database migrations...");
    sqlx::migrate!("../../sql")
        .run(pool)
        .await
        .map_err(|e| crate::AppError::DatabaseError(format!("Migration failed: {e}")))?;
    info!("Migrations completed successfully");
    Ok(())
}
