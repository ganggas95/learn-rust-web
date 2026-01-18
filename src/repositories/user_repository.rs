use crate::domain::user::UserRepository;
use crate::domain::{User, UserResult};
use crate::infrastructure::error::AppError;
use async_trait::async_trait;
use sqlx::PgPool;

pub struct PostgresUserRepository {
    pub pool: PgPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, username: &str, password_hash: &str) -> Result<User, AppError> {
        let created_user = sqlx::query_as!(
            User,
            "INSERT INTO users(username, password_hash) VALUES ($1, $2) RETURNING id, username",
            username,
            password_hash,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|err| {
            if let sqlx::Error::Database(db_err) = &err {
                if db_err.is_unique_violation() {
                    return AppError::Conflict("Username already exists".into());
                }
            }
            AppError::Internal("Failed to create user".into())
        })?;
        Ok(created_user)
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<UserResult>, AppError> {
        let result = sqlx::query_as!(
            UserResult,
            "SELECT id, username, password_hash FROM users WHERE username = $1",
            username
        );
        let user = result
            .fetch_optional(&self.pool)
            .await
            .map_err(|_| AppError::Internal("Failed to query user by username".into()))?;
        Ok(user)
    }

    async fn find_by_id(&self, user_id: i32) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as!(
            User,
            "SELECT id, username FROM users WHERE id = $1",
            user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| AppError::Internal("Failed to query user".into()))?;
        Ok(Some(user))
    }
}
