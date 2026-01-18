use async_trait::async_trait;
use serde::Serialize;
use sqlx;

use crate::infrastructure::error::AppError;

#[derive(Serialize, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct UserResult {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

#[async_trait]
pub trait UserRepository {
    async fn create_user(&self, username: &str, password_hash: &str) -> Result<User, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<UserResult>, AppError>;
    async fn find_by_id(&self, user_id: i32) -> Result<Option<User>, AppError>;
}
