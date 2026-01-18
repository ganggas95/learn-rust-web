use crate::config::AppConfig;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_db_pool(config: &AppConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await
}
