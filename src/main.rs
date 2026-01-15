use core::fmt;
use std::env;

use actix_cors::Cors;
use actix_web::{web, App, FromRequest, HttpResponse, HttpServer, ResponseError};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use dotenv::dotenv;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, PgPool};

use std::future::{ready, Ready};

struct AppState {
    db_pool: PgPool,
    jwt_secret: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct User {
    id: i32,
    username: String,
}

#[derive(Serialize, sqlx::FromRow)]
struct UserResult {
    id: i32,
    username: String,
    password_hash: String,
}

#[derive(Deserialize)]
struct RegisterPayload {
    username: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    exp: usize,
}

struct JwtMiddleware {
    claims: Claims,
}

#[derive(Debug)]
enum AppError {
    Internal(String),
    Conflict(String),
    Unauthorized(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Internal(msg) => write!(f, "Internal server error: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Internal(msg) => HttpResponse::InternalServerError().body(msg.clone()),
            AppError::Conflict(msg) => HttpResponse::Conflict().body(msg.clone()),
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().body(msg.clone()),
        }
    }
}

impl FromRequest for JwtMiddleware {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        // Get Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => return ready(Err(AppError::Unauthorized(
                "Missing Authorization header".to_string(),
            ))),
        };
        // Get Authentication from header
        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return ready(Err(AppError::Unauthorized("Invalid Authorization header".to_string())))
            }
        };
        if !auth_str.starts_with("Bearer ") {
            return ready(Err(AppError::Unauthorized("Invalid Authorization header".to_string())));
        }
        // Get token from header
        let token = &auth_str["Bearer ".len()..];
        let app_state = req.app_data::<web::Data<AppState>>().unwrap();
        // Validate token with secret key
        let decoding_key = DecodingKey::from_secret(app_state.jwt_secret.as_ref());
        let validation = Validation::default();
        match decode::<Claims>(&token, &decoding_key, &validation) {
            Ok(decoded) => ready(Ok(JwtMiddleware {
                claims: decoded.claims,
            })),
            Err(_) => ready(Err(AppError::Unauthorized("Invalid token".to_string()))),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let max_connection = env::var("MAX_CONNECTIONS").unwrap_or("5".to_string());
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(max_connection.parse().unwrap_or(5))
        .connect(&db_url)
        .await
        .expect("Failed to connect to database");

    println!("Server running on http://localhost:8080");
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
                    .route("/register", web::post().to(register_handler))
                    .route("/login", web::post().to(login_handler))
                    .route("/profile", web::get().to(profile_handler)),
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn generate_token(user: UserResult, jwt_secret: &str) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::days(1)).timestamp() as usize;
    let claims = Claims { sub: user.id, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::Internal("Failed to create token".to_string()))?;
    Ok(token)
}

async fn register_handler(
    state: web::Data<AppState>,
    payload: web::Json<RegisterPayload>,
) -> Result<HttpResponse, AppError> {
    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| AppError::Internal("Failed to hash password".to_string()))?;
    let created_user = sqlx::query_as!(
        User,
        "INSERT INTO users(username, password_hash) VALUES ($1, $2) RETURNING id, username",
        payload.username,
        password_hash,
    )
    .fetch_one(&state.db_pool)
    .await;

    match created_user {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(AppError::Conflict("Username already exists".to_string()))
        }
        Err(_) => Err(AppError::Internal("Failed to crate user".to_string())),
    }
}

async fn login_handler(
    payload: web::Json<LoginPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let result = sqlx::query_as!(
        UserResult,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        payload.username
    )
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|_| AppError::Internal("Failed to login".to_string()))?;

    match result {
        Some(user) => {
            let is_password_valid = verify(&payload.password, &user.password_hash)
                .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;
            if is_password_valid {
                let token = generate_token(user, &state.jwt_secret)
                    .map_err(|_| AppError::Internal("Failed to create token".to_string()))?;
                Ok(HttpResponse::Ok().json(LoginResponse {
                    access_token: token,
                }))
            } else {
                Err(AppError::Unauthorized(
                    "Invalid username or password".to_string(),
                ))
            }
        }
        None => Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        )),
    }
}

async fn profile_handler(
    jwt: JwtMiddleware,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user_id = jwt.claims.sub;
    let user = sqlx::query_as!(
        User,
        "SELECT id, username FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(&state.db_pool)
    .await
    .map_err(|_| AppError::Internal("Failed to get user profile".to_string()))?;
    Ok(HttpResponse::Ok().json(user))
}
