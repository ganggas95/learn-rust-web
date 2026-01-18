use crate::domain::{User, UserResult};
use sqlx::PgPool;

pub async fn create_user(
    db_pool: &PgPool,
    username: &str,
    password_hash: &str,
) -> Result<User, sqlx::Error> {
    let created_user = sqlx::query_as!(
        User,
        "INSERT INTO users(username, password_hash) VALUES ($1, $2) RETURNING id, username",
        username,
        password_hash,
    )
    .fetch_one(db_pool)
    .await;
    Ok(created_user?)
}

pub async fn find_user_by_username(
    pool: &PgPool,
    username: &str,
) -> Result<Option<UserResult>, sqlx::Error> {
    let result = sqlx::query_as!(
        UserResult,
        "SELECT id, username, password_hash FROM users WHERE username = $1",
        username
    );
    let user = result.fetch_optional(pool).await?;
    Ok(user)
}

pub async fn find_user_by_id(db_pool: &PgPool, user_id: i32) -> Result<User, sqlx::Error> {
    let user = sqlx::query_as!(
        User,
        "SELECT id, username FROM users WHERE id = $1",
        user_id
    )
    .fetch_one(db_pool)
    .await;
    Ok(user?)
}
