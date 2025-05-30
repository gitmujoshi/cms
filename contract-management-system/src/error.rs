//! Contract Management System - Error Handling Module
//! 
//! This module provides comprehensive error handling for the Contract Management System.
//! It defines two main error types:
//! - AppError: Application-level errors for internal use
//! - ApiError: HTTP API-specific errors for client responses
//!
//! Features:
//! - Structured error types for different error scenarios
//! - Automatic conversion between error types
//! - Standardized error response format
//! - Integration with actix-web's error handling system
//! - Support for blockchain and contract-specific errors
//!
//! Error Categories:
//! - Database errors
//! - Authentication errors
//! - Validation errors
//! - Resource not found errors
//! - Blockchain interaction errors
//! - Contract state errors
//! - Signature verification errors
//! - Internal server errors
//!
//! Usage:
//! 1. Use AppError for internal error handling
//! 2. Use ApiError for HTTP API responses
//! 3. Convert between error types using From implementations
//! 4. Return errors using the Result type alias
//!
//! Author: Contract Management System Team
//! License: MIT

use actix_web::{HttpResponse, ResponseError};
use sea_orm::DbErr;
use serde::Serialize;
use std::fmt;
use argon2::password_hash;
use thiserror::Error;

/// Application-level error types
/// 
/// This enum defines all possible error types that can occur in the application,
/// including database errors, authentication issues, validation failures, and more.
/// Each variant includes a descriptive message to help with debugging and user feedback.
#[derive(Error, Debug)]
pub enum AppError {
    /// Database operation errors (e.g., connection issues, query failures)
    #[error("Database error: {0}")]
    DatabaseError(#[from] sea_orm::DbErr),

    /// Authentication and authorization related errors
    #[error("Authentication error: {0}")]
    AuthError(String),

    /// Input validation errors (e.g., invalid data format, missing required fields)
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// Resource not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Blockchain interaction errors (e.g., transaction failures, network issues)
    #[error("Blockchain error: {0}")]
    BlockchainError(String),

    /// Contract state-related errors (e.g., invalid state transitions)
    #[error("Contract state error: {0}")]
    ContractStateError(String),

    /// Digital signature verification errors
    #[error("Signature verification error: {0}")]
    SignatureError(String),

    /// Unexpected internal errors
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Type alias for Result that uses AppError as the error type
pub type Result<T> = std::result::Result<T, AppError>;

// Implement conversion from ethers provider errors to AppError
impl From<ethers::prelude::ProviderError> for AppError {
    fn from(err: ethers::prelude::ProviderError) -> Self {
        AppError::BlockchainError(err.to_string())
    }
}

// Implement conversion from ethers contract errors to AppError
impl From<ethers::prelude::ContractError<ethers::providers::Provider<ethers::providers::Http>>> for AppError {
    fn from(err: ethers::prelude::ContractError<ethers::providers::Provider<ethers::providers::Http>>) -> Self {
        AppError::BlockchainError(err.to_string())
    }
}

// Implement conversion from ethers wallet errors to AppError
impl From<ethers::signers::WalletError> for AppError {
    fn from(err: ethers::signers::WalletError) -> Self {
        AppError::BlockchainError(format!("Wallet error: {}", err))
    }
}

/// API-specific error types
/// 
/// This enum defines error types specifically for HTTP API responses,
/// including appropriate HTTP status codes and error messages.
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

/// Standardized error response structure for API endpoints
/// 
/// This struct defines the format of error responses returned to API clients,
/// including an error code and a human-readable message.
#[derive(Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
}

/// Implement Display trait for ApiError to provide human-readable error messages
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

/// Implement ResponseError trait for ApiError to handle HTTP error responses
impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let error = ErrorResponse {
            code: self.code(),
            message: self.to_string(),
        };

        // Map each error type to appropriate HTTP status code and response
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
    /// Get the error code for the error type
    /// 
    /// Returns a string representation of the error code that can be used
    /// by clients to programmatically handle different error types.
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

// Implement conversion from database errors to ApiError
impl From<DbErr> for ApiError {
    fn from(err: DbErr) -> Self {
        ApiError::DatabaseError(err)
    }
}

// Implement conversion from password hash errors to ApiError
impl From<password_hash::Error> for ApiError {
    fn from(err: password_hash::Error) -> Self {
        ApiError::PasswordHashError(err)
    }
} 