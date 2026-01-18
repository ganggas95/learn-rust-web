use actix_web::{HttpResponse, ResponseError};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Internal(String),
    Conflict(String),
    Unauthorized(String),
    NotFound(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Internal(msg) => write!(f, "Internal server error: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Internal(msg) => HttpResponse::InternalServerError().body(msg.clone()),
            AppError::Conflict(msg) => HttpResponse::Conflict().body(msg.clone()),
            AppError::Unauthorized(msg) => HttpResponse::Unauthorized().body(msg.clone()),
            AppError::NotFound(msg) => HttpResponse::NotFound().body(msg.clone()),
        }
    }
}
