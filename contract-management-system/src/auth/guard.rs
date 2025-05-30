use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::future::Future;
use std::pin::Pin;

use crate::error::ApiError;

/// JWT Claims structure for authentication
/// 
/// Contains the essential information stored in the JWT token:
/// - sub: Subject (user ID)
/// - exp: Expiration time
/// - role: User role for authorization
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub role: String,
}

/// Authentication guard structure
/// 
/// Holds the authenticated user's information that will be
/// available in route handlers after successful authentication.
pub struct AuthGuard {
    pub user_id: Uuid,
    pub role: String,
}

/// Authentication middleware factory
/// 
/// Creates new instances of the authentication middleware service.
pub struct AuthMiddleware;

/// Implementation of the Transform trait for AuthMiddleware
/// 
/// This allows the middleware to be used with actix-web's middleware system.
impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Create a new instance of the authentication middleware service
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

/// Authentication middleware service
/// 
/// Handles the actual authentication logic for each request.
pub struct AuthMiddlewareService<S> {
    service: S,
}

/// Implementation of the Service trait for AuthMiddlewareService
/// 
/// Processes incoming requests and validates JWT tokens.
impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Forward the ready state to the inner service
    forward_ready!(service);

    /// Process an incoming request
    /// 
    /// Validates the JWT token in the Authorization header and attaches
    /// the authenticated user's information to the request.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        
        // Process the authorization header
        let auth_result = match auth_header {
            Some(header) => {
                let auth_str = header.to_str().unwrap_or("");
                // Verify Bearer token format
                if !auth_str.starts_with("Bearer ") {
                    Err(ApiError::Unauthorized)
                } else {
                    // Extract and validate the token
                    let token = &auth_str[7..];
                    let key = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
                    
                    // Decode and validate the JWT token
                    match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(key.as_bytes()),
                        &Validation::default(),
                    ) {
                        Ok(token_data) => {
                            // Create and attach the auth guard to the request
                            let auth_guard = AuthGuard {
                                user_id: token_data.claims.sub,
                                role: token_data.claims.role,
                            };
                            req.extensions_mut().insert(auth_guard);
                            Ok(())
                        }
                        Err(_) => Err(ApiError::Unauthorized),
                    }
                }
            }
            None => Err(ApiError::Unauthorized),
        };

        // Process the request with the inner service
        let fut = self.service.call(req);
        
        // Handle the authentication result
        Box::pin(async move {
            match auth_result {
                Ok(_) => fut.await,
                Err(e) => Err(e.into()),
            }
        })
    }
} 