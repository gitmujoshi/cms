use actix_web::dev::ServiceRequest;
use actix_web::error::ErrorUnauthorized;
use actix_web::{Error, HttpMessage};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
    pub role: String,
}

pub struct Auth {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl Auth {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn generate_token(&self, user_id: Uuid, role: String) -> Result<String, Error> {
        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id,
            exp: now + 24 * 3600, // 24 hours from now
            iat: now,
            role,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ErrorUnauthorized(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, Error> {
        let validation = Validation::default();
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| ErrorUnauthorized(e.to_string()))
    }
}

pub struct AuthMiddleware {
    auth: Auth,
}

impl AuthMiddleware {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            auth: Auth::new(secret),
        }
    }

    pub fn validate(&self, req: ServiceRequest) -> Ready<Result<ServiceRequest, Error>> {
        let auth_header = req.headers().get("Authorization");
        
        match auth_header {
            Some(auth_str) => {
                let auth_str = auth_str.to_str().unwrap_or("");
                if !auth_str.starts_with("Bearer ") {
                    return ready(Err(ErrorUnauthorized("Invalid authorization header")));
                }

                let token = &auth_str[7..];
                match self.auth.validate_token(token) {
                    Ok(claims) => {
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

    #[test]
    fn test_token_lifecycle() {
        let secret = b"test_secret";
        let auth = Auth::new(secret);
        let user_id = Uuid::new_v4();
        let role = "admin".to_string();

        // Generate token
        let token = auth.generate_token(user_id, role.clone()).unwrap();

        // Validate token
        let claims = auth.validate_token(&token).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, role);
    }
} 