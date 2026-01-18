mod api;
mod config;
mod domain;
mod infrastructure;
mod repositories;
mod services;
mod startup;
use crate::config::AppConfig;
use crate::startup::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env();
    run(config).await
}
