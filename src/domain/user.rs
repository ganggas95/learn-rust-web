use serde::Serialize;
use sqlx;

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
