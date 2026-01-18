use serde::{Deserialize, Serialize};
use std::future::{Ready, ready};
use crate::infrastructure::error::AppError;
use crate::infrastructure::state::AppState;
use actix_web::{FromRequest, web};
use jsonwebtoken::{decode, DecodingKey, Validation};


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    pub exp: usize,
}

pub struct JwtMiddleware {
    pub claims: Claims,
}

impl FromRequest for JwtMiddleware {
    type Error = AppError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Get Authorization header
        let auth_header = match req.headers().get("Authorization") {
            Some(header) => header,
            None => {
                return ready(Err(AppError::Unauthorized(
                    "Missing Authorization header".to_string(),
                )));
            }
        };
        // Get Authentication from header
        let auth_str = match auth_header.to_str() {
            Ok(s) => s,
            Err(_) => {
                return ready(Err(AppError::Unauthorized(
                    "Invalid Authorization header".to_string(),
                )));
            }
        };
        if !auth_str.starts_with("Bearer ") {
            return ready(Err(AppError::Unauthorized(
                "Invalid Authorization header".to_string(),
            )));
        }
        // Get token from header
        let token = &auth_str["Bearer ".len()..];
        let app_state = req.app_data::<web::Data<AppState>>().unwrap();
        // Validate token with secret key
        let decoding_key = DecodingKey::from_secret(app_state.jwt_secret.as_ref());
        let validation = Validation::default();
        match decode::<Claims>(&token, &decoding_key, &validation) {
            Ok(decoded) => ready(Ok(JwtMiddleware {
                claims: decoded.claims,
            })),
            Err(_) => ready(Err(AppError::Unauthorized("Invalid token".to_string()))),
        }
    }
}
