
use crate::services::user_service;
use crate::{infrastructure::state::AppState, repositories::user_repository::PostgresUserRepository};
use crate::infrastructure::error::AppError;
use crate::infrastructure::jwt::JwtMiddleware;
use actix_web::{HttpResponse, web};

// Profile Handler
pub async fn profile_handler(
    jwt: JwtMiddleware,
    state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    let repo = PostgresUserRepository {
        pool: state.db_pool.clone(),
    };
    let user_id = jwt.claims.sub;
    let user = user_service::find_user_by_id(&repo, user_id).await?;
    Ok(HttpResponse::Ok().json(user))
}
