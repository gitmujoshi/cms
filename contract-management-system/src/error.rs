use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;
use serde::Serialize;
use std::fmt;
use argon2::password_hash;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    #[error("Contract state error: {0}")]
    ContractStateError(String),

    #[error("Signature verification error: {0}")]
    SignatureError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

impl From<ethers::prelude::ProviderError> for AppError {
    fn from(err: ethers::prelude::ProviderError) -> Self {
        AppError::BlockchainError(err.to_string())
    }
}

impl From<ethers::prelude::ContractError<ethers::providers::Provider<ethers::providers::Http>>> for AppError {
    fn from(err: ethers::prelude::ContractError<ethers::providers::Provider<ethers::providers::Http>>) -> Self {
        AppError::BlockchainError(err.to_string())
    }
}

impl From<ethers::signers::WalletError> for AppError {
    fn from(err: ethers::signers::WalletError) -> Self {
        AppError::BlockchainError(format!("Wallet error: {}", err))
    }
}

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    DatabaseError(DbErr),
    ValidationError(String),
    InvalidCredentials,
    AccountInactive,
    Forbidden,
    InvalidContractState,
    PasswordHashError(password_hash::Error),
    InternalServerError(String),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::NotFound => write!(f, "Resource not found"),
            ApiError::DatabaseError(e) => write!(f, "Database error: {}", e),
            ApiError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ApiError::InvalidCredentials => write!(f, "Invalid credentials"),
            ApiError::AccountInactive => write!(f, "Account is inactive"),
            ApiError::Forbidden => write!(f, "Access forbidden"),
            ApiError::InvalidContractState => write!(f, "Invalid contract state for this operation"),
            ApiError::PasswordHashError(e) => write!(f, "Password hashing error: {}", e),
            ApiError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error = ErrorResponse {
            code: self.code(),
            message: self.to_string(),
        };

        match self {
            ApiError::NotFound => HttpResponse::NotFound().json(error),
            ApiError::DatabaseError(_) => HttpResponse::InternalServerError().json(error),
            ApiError::ValidationError(_) => HttpResponse::BadRequest().json(error),
            ApiError::InvalidCredentials => HttpResponse::Unauthorized().json(error),
            ApiError::AccountInactive => HttpResponse::Forbidden().json(error),
            ApiError::Forbidden => HttpResponse::Forbidden().json(error),
            ApiError::InvalidContractState => HttpResponse::BadRequest().json(error),
            ApiError::PasswordHashError(_) => HttpResponse::InternalServerError().json(error),
            ApiError::InternalServerError(_) => HttpResponse::InternalServerError().json(error),
        }
    }
}

impl ApiError {
    fn code(&self) -> String {
        match self {
            ApiError::NotFound => "NOT_FOUND",
            ApiError::DatabaseError(_) => "DATABASE_ERROR",
            ApiError::ValidationError(_) => "VALIDATION_ERROR",
            ApiError::InvalidCredentials => "INVALID_CREDENTIALS",
            ApiError::AccountInactive => "ACCOUNT_INACTIVE",
            ApiError::Forbidden => "FORBIDDEN",
            ApiError::InvalidContractState => "INVALID_CONTRACT_STATE",
            ApiError::PasswordHashError(_) => "PASSWORD_HASH_ERROR",
            ApiError::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
        }
        .to_string()
    }
}

impl From<DbErr> for ApiError {
    fn from(err: DbErr) -> Self {
        ApiError::DatabaseError(err)
    }
}

impl From<password_hash::Error> for ApiError {
    fn from(err: password_hash::Error) -> Self {
        ApiError::PasswordHashError(err)
    }
} 