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

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub role: String,
}

pub struct AuthGuard {
    pub user_id: Uuid,
    pub role: String,
}

pub struct AuthMiddleware;

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

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization");
        
        let auth_result = match auth_header {
            Some(header) => {
                let auth_str = header.to_str().unwrap_or("");
                if !auth_str.starts_with("Bearer ") {
                    Err(ApiError::Unauthorized)
                } else {
                    let token = &auth_str[7..];
                    let key = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret".to_string());
                    
                    match decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(key.as_bytes()),
                        &Validation::default(),
                    ) {
                        Ok(token_data) => {
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

        let fut = self.service.call(req);
        
        Box::pin(async move {
            match auth_result {
                Ok(_) => fut.await,
                Err(e) => Err(e.into()),
            }
        })
    }
} 