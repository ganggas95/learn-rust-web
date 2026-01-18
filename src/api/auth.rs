use crate::domain::{AppError, AppState, LoginPayload, LoginResponse, RegisterPayload};
use crate::repositories::{create_user, find_user_by_username};
use crate::services::auth_service;
use actix_web::{HttpResponse, web};
use bcrypt::{DEFAULT_COST, hash, verify};
use sqlx;

// Register Handler
pub async fn register_handler(
    state: web::Data<AppState>,
    payload: web::Json<RegisterPayload>,
) -> Result<HttpResponse, AppError> {
    let password_hash = hash(&payload.password, DEFAULT_COST)
        .map_err(|_| AppError::Internal("Failed to hash password".to_string()))?;
    let created_user_result = create_user(&state.db_pool, &payload.username, &password_hash).await;

    match created_user_result {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            Err(AppError::Conflict("Username already exists".to_string()))
        }
        Err(_) => Err(AppError::Internal("Failed to crate user".to_string())),
    }
}

// Login Handler
pub async fn login_handler(
    payload: web::Json<LoginPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let result = find_user_by_username(&state.db_pool, &payload.username)
        .await
        .map_err(|_| AppError::Internal("Failed to login".to_string()))?;

    match result {
        Some(user) => {
            let is_password_valid = verify(&payload.password, &user.password_hash)
                .map_err(|_| AppError::Internal("Password verification failed".to_string()))?;
            if is_password_valid {
                let token = auth_service::generate_token(user, &state.jwt_secret)
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
