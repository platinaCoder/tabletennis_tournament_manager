use sqlx_postgres::{PgConnectOptions, PgPool, PgPoolOptions};

use crate::backend::server::config::ServerConfig;

pub async fn connect(config: &ServerConfig) -> Result<PgPool, sqlx::Error> {
    let options = config.database_url.parse::<PgConnectOptions>()?;
    PgPoolOptions::new()
        .max_connections(config.database_max_connections.get())
        .connect_with(options)
        .await
}

#[cfg(test)]
pub async fn migrate_test_database(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("server/migrations");
    sqlx::migrate::Migrator::new(path.as_path())
        .await?
        .run(pool)
        .await
}
