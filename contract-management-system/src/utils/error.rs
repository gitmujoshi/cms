use actix_web::{error::ResponseError, HttpResponse};
use sea_orm::DbErr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] DbErr),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Enclave error: {0}")]
    Enclave(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal server error"
                }))
            }
            AppError::Auth(msg) => HttpResponse::Unauthorized().json(json!({
                "error": msg
            })),
            AppError::Validation(msg) => HttpResponse::BadRequest().json(json!({
                "error": msg
            })),
            AppError::NotFound(msg) => HttpResponse::NotFound().json(json!({
                "error": msg
            })),
            AppError::Enclave(msg) => HttpResponse::InternalServerError().json(json!({
                "error": msg
            })),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                HttpResponse::InternalServerError().json(json!({
                    "error": "Internal server error"
                }))
            }
        }
    }
}

pub type AppResult<T> = Result<T, AppError>; 