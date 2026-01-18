use sqlx::PgPool;
pub mod auth;
pub mod error;
pub mod jwt;
pub mod user;

pub use auth::{LoginPayload, LoginResponse, RegisterPayload};
pub use error::AppError;
pub use user::{User, UserResult};

pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret: String,
}
