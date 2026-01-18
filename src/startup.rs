use crate::api;
use crate::config::AppConfig;
use crate::infrastructure::db::create_db_pool;
use crate::infrastructure::state::AppState;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};

pub async fn run(config: AppConfig) -> std::io::Result<()> {
    let db_pool = create_db_pool(&config)
        .await
        .expect("Failed to connect to database");

    let host = config.server_host.clone();
    let port = config.server_port;
    let jwt_secret = config.jwt_secret.clone();
    println!("Server running on http://{}:{}", host, port);
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
                jwt_secret: jwt_secret.clone(),
            }))
            .service(
                web::scope("/api")
                    .route("/register", web::post().to(api::auth::register_handler))
                    .route("/login", web::post().to(api::auth::login_handler))
                    .route("/profile", web::get().to(api::users::profile_handler)),
            )
    })
    .bind((host, port))?
    .run()
    .await
}
