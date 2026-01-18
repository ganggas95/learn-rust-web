use crate::domain::AppState;
use crate::domain::error::AppError;
use crate::domain::jwt::JwtMiddleware;
use crate::repositories::user_repository::find_user_by_id;
use actix_web::{HttpResponse, web};

// Profile Handler
pub async fn profile_handler(
    jwt: JwtMiddleware,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let user_id = jwt.claims.sub;
    let user = find_user_by_id(&state.db_pool, user_id)
        .await
        .map_err(|_| AppError::Internal("Failed to get user profile".to_string()))?;
    Ok(HttpResponse::Ok().json(user))
}
