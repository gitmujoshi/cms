//! Contract Management System - Authentication Module
//! 
//! This module provides JWT-based authentication for the Contract Management System.
//! It implements a complete authentication system with:
//! - JWT token generation and validation
//! - User authentication middleware
//! - Role-based access control
//!
//! Components:
//! - Claims: JWT token payload structure
//! - Auth: Core authentication service
//! - AuthMiddleware: Request authentication middleware
//!
//! Features:
//! - Secure token generation with expiration
//! - Token validation and verification
//! - User role management
//! - Integration with actix-web middleware system
//! - Extensible authentication guard system
//!
//! Security Considerations:
//! - Tokens expire after 24 hours
//! - Secure secret key management
//! - Role-based access control
//! - Bearer token authentication
//!
//! Usage:
//! 1. Initialize Auth with a secret key
//! 2. Generate tokens for authenticated users
//! 3. Use AuthMiddleware to protect routes
//! 4. Access user information in route handlers
//!
//! Author: Contract Management System Team
//! License: MIT

// Import required dependencies
use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

/// JWT Claims structure
/// 
/// Represents the data contained within a JWT token:
/// - sub: Subject (user ID)
/// - exp: Expiration time
/// - iat: Issued at time
/// - role: User role for authorization
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

/// Authentication service for JWT token management
/// 
/// Handles token generation and validation using a secret key.
/// Provides methods for creating new tokens and verifying existing ones.
pub struct Auth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Auth {
    /// Create a new Auth instance with the provided secret key
    /// 
    /// # Arguments
    /// * `secret` - The secret key used for signing and verifying tokens
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    /// Generate a new JWT token for a user
    /// 
    /// # Arguments
    /// * `user_id` - The UUID of the user
    /// * `role` - The user's role for authorization
    /// 
    /// # Returns
    /// A Result containing either the generated token or an error
    pub fn generate_token(&self, user_id: Uuid, role: String) -> Result<String, Error> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id,
            exp: now + 24 * 3600, // Token expires in 24 hours
            iat: now,
            role,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ErrorUnauthorized(e.to_string()))
    }

    /// Validate a JWT token and extract its claims
    /// 
    /// # Arguments
    /// * `token` - The JWT token to validate
    /// 
    /// # Returns
    /// A Result containing either the token claims or an error
    pub fn validate_token(&self, token: &str) -> Result<Claims, Error> {
        let validation = Validation::default();
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| ErrorUnauthorized(e.to_string()))
    }
}

/// Middleware for handling authentication in HTTP requests
/// 
/// Validates JWT tokens in the Authorization header and attaches
/// the claims to the request for use in route handlers.
pub struct AuthMiddleware {
    auth: Auth,
}

impl AuthMiddleware {
    /// Create a new AuthMiddleware instance
    /// 
    /// # Arguments
    /// * `secret` - The secret key used for token validation
    pub fn new(secret: &[u8]) -> Self {
        Self {
            auth: Auth::new(secret),
        }
    }

    /// Validate the Authorization header and process the request
    /// 
    /// # Arguments
    /// * `req` - The incoming service request
    /// 
    /// # Returns
    /// A Ready future containing either the processed request or an error
    pub fn validate(&self, req: ServiceRequest) -> Ready<Result<ServiceRequest, Error>> {
        let auth_header = req.headers().get("Authorization");
        
        match auth_header {
            Some(auth_str) => {
                let auth_str = auth_str.to_str().unwrap_or("");
                // Check for Bearer token format
                if !auth_str.starts_with("Bearer ") {
                    return ready(Err(ErrorUnauthorized("Invalid authorization header")));
                }

                // Extract and validate the token
                let token = &auth_str[7..];
                match self.auth.validate_token(token) {
                    Ok(claims) => {
                        // Attach claims to request for use in handlers
                        req.extensions_mut().insert(claims);
                        ready(Ok(req))
                    }
                    Err(e) => ready(Err(e)),
                }
            }
            None => ready(Err(ErrorUnauthorized("Missing authorization header"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the complete token lifecycle: generation and validation
    #[test]
    fn test_token_lifecycle() {
        let secret = b"test_secret";
        let auth = Auth::new(secret);
        let user_id = Uuid::new_v4();
        let role = "admin".to_string();

        // Generate token
        let token = auth.generate_token(user_id, role.clone()).unwrap();

        // Validate token and verify claims
        let claims = auth.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, role);
    }
} 