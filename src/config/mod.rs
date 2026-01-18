use dotenv::dotenv;
use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub jwt_secret: String,
    pub max_connections: u32,
    pub server_host: String,
    pub server_port: u16,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenv().ok();

        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let max_connection = env::var("MAX_CONNECTIONS").unwrap_or("5".to_string());
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let server_host = env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT").unwrap_or("8080".to_string());
        Self {
            database_url: db_url,
            max_connections: max_connection.parse().unwrap_or(5),
            jwt_secret,
            server_host,
            server_port: server_port.parse().unwrap_or(8080),
        }
    }
}
