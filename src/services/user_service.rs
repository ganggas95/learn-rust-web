use crate::domain::User;
use bcrypt::{DEFAULT_COST, hash, verify};

use crate::{
    domain::{LoginPayload, LoginResponse, user::UserRepository},
    infrastructure::error::AppError,
    services::auth_service,
};

pub async fn create_user<R: UserRepository + Sync>(
    repo: &R,
    username: &str,
    password: &str,
) -> Result<User, AppError> {
    let password_hash = hash(password, DEFAULT_COST)
        .map_err(|_| AppError::Internal("Failed to hash password".to_string()))?;
    let user = repo.create_user(username, &password_hash).await?;
    Ok(user)
}

pub async fn login_user<R: UserRepository + Sync>(
    repo: &R,
    payload: &LoginPayload,
    jwt_secret: &str,
) -> Result<LoginResponse, AppError> {
    let user_pot = repo.find_by_username(&payload.username).await?;

    let user = match user_pot {
        Some(u) => u,
        None => {
            return Err(AppError::Unauthorized(
                "Invalid username or password".into(),
            ));
        }
    };
    let is_valid_password = verify(&payload.password, &user.password_hash)
        .map_err(|_| AppError::Internal("Password verification failed".into()))?;
    if !is_valid_password {
        return Err(AppError::Unauthorized(
            "Invalid username or password".into(),
        ))?;
    }

    let token = auth_service::generate_token(user, jwt_secret)?;
    Ok(LoginResponse {
        access_token: token,
    })
}

pub async fn find_user_by_id<R: UserRepository + Sync>(
    repo: &R,
    user_id: i32,
) -> Result<User, AppError> {
    let user = repo.find_by_id(user_id).await?;
    match user {
        Some(u) => Ok(u),
        None => {
            return Err(AppError::NotFound(
                "User not found".into(),
            ));
        }
    }
}
