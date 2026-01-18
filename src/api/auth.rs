use crate::domain::{LoginPayload, RegisterPayload};
use crate::infrastructure::error::AppError;
use crate::infrastructure::state::AppState;
use crate::repositories::user_repository::PostgresUserRepository;
use crate::services::user_service;
use actix_web::{HttpResponse, web};

// Register Handler
pub async fn register_handler(
    state: web::Data<AppState>,
    payload: web::Json<RegisterPayload>,
) -> Result<HttpResponse, AppError> {
    let repo = PostgresUserRepository {
        pool: state.db_pool.clone(),
    };
    let user = user_service::create_user(&repo, &payload.username, &payload.password).await?;
    Ok(HttpResponse::Ok().json(user))
}

// Login Handler
pub async fn login_handler(
    payload: web::Json<LoginPayload>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let repo = PostgresUserRepository {
        pool: state.db_pool.clone(),
    };
    let login_response = user_service::login_user(&repo, &payload, &state.jwt_secret).await?;
    Ok(HttpResponse::Ok().json(login_response))
}
