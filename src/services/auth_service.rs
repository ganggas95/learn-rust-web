// mod domain;
use crate::AppError;
use crate::domain::jwt::Claims;
use crate::UserResult;
use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};

pub fn generate_token(user: UserResult, jwt_secret: &str) -> Result<String, AppError> {
    let exp = (Utc::now() + Duration::days(1)).timestamp() as usize;
    let claims = Claims { sub: user.id, exp };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| AppError::Internal("Failed to create token".to_string()))?;
    Ok(token)
}
