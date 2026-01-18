mod api;
mod domain;
mod services;
mod config;
mod repositories;
use crate::domain::AppState;
use crate::domain::UserResult;
use crate::domain::error::AppError;
use crate::config::AppConfig;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use sqlx::postgres::PgPoolOptions;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env();
    let db_pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.db_url)
        .await
        .expect("Failed to connect to database");

    println!("Server running on http://{}:{}", config.server_host, config.server_port);
    HttpServer::new(move || {
        // For development, allow any origin, method, and header
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(AppState {
                db_pool: db_pool.clone(),
                jwt_secret: config.jwt_secret.clone(),
            }))
            .service(
                web::scope("/api")
                    .route("/register", web::post().to(api::auth::register_handler))
                    .route("/login", web::post().to(api::auth::login_handler))
                    .route("/profile", web::get().to(api::users::profile_handler)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
