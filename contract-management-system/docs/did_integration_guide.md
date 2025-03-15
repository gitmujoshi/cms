# Step-by-Step DID Integration Guide

## Prerequisites

1. **Dependencies**
   Add these to your `Cargo.toml`:
   ```toml
   [dependencies]
   # Core dependencies
   serde = { version = "1.0", features = ["derive"] }
   tokio = { version = "1.0", features = ["full"] }
   actix-web = "4.0"
   
   # DID-specific
   did-resolver = "0.5"
   did-jwt = "0.2"
   
   # Cryptography
   rand = "0.8"
   hex = "0.4"
   sha2 = "0.10"
   
   # Storage
   redis = { version = "0.22", features = ["tokio-comp"] }
   
   # Utilities
   chrono = { version = "0.4", features = ["serde"] }
   uuid = { version = "1.0", features = ["v4", "serde"] }
   thiserror = "1.0"
   tracing = "0.1"
   ```

## Step 1: Basic Setup

1. **Create Directory Structure**
   ```bash
   mkdir -p src/auth/did
   touch src/auth/did/mod.rs
   touch src/auth/did/challenge.rs
   touch src/auth/did/resolver.rs
   touch src/auth/did/verification.rs
   touch src/auth/middleware.rs
   touch src/auth/token.rs
   ```

2. **Configure Module Structure**
   In `src/auth/mod.rs`:
   ```rust
   pub mod did;
   pub mod middleware;
   pub mod token;

   pub use did::{AuthChallenge, DIDResolver};
   pub use middleware::DIDAuthMiddleware;
   pub use token::{generate_jwt_token, JWTClaims};
   ```

## Step 2: Implement Core Components

1. **Challenge Generation (`src/auth/did/challenge.rs`)**
   ```rust
   use chrono::{DateTime, Duration, Utc};
   use serde::{Deserialize, Serialize};
   use uuid::Uuid;

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AuthChallenge {
       pub challenge_id: String,
       pub did: String,
       pub nonce: String,
       pub timestamp: DateTime<Utc>,
       pub expiration: DateTime<Utc>,
   }

   impl AuthChallenge {
       pub fn new(did: String) -> Self {
           let now = Utc::now();
           Self {
               challenge_id: Uuid::new_v4().to_string(),
               did,
               nonce: generate_secure_nonce(),
               timestamp: now,
               expiration: now + Duration::minutes(5),
           }
       }
   }

   // Add this to your configuration
   pub struct ChallengeConfig {
       pub timeout_seconds: u64,
       pub nonce_bytes: usize,
   }
   ```

2. **DID Resolution (`src/auth/did/resolver.rs`)**
   ```rust
   use async_trait::async_trait;
   use std::collections::HashMap;

   #[async_trait]
   pub trait DIDResolver: Send + Sync {
       async fn resolve(&self, did: &str) -> Result<Document, Error>;
   }

   pub struct MultiResolver {
       resolvers: HashMap<String, Box<dyn DIDResolver>>,
       cache: Cache,
   }

   // Add configuration
   pub struct ResolverConfig {
       pub cache_ttl: Duration,
       pub timeout: Duration,
       pub max_redirects: u32,
   }
   ```

## Step 3: Storage Implementation

1. **Redis Store Setup**
   ```rust
   pub struct RedisConfig {
       pub url: String,
       pub prefix: String,
       pub pool_size: u32,
   }

   pub struct RedisStore {
       client: redis::Client,
       config: RedisConfig,
   }

   impl RedisStore {
       pub async fn new(config: RedisConfig) -> Result<Self, Error> {
           let client = redis::Client::open(config.url.as_str())?;
           Ok(Self { client, config })
       }
   }
   ```

2. **Challenge Storage Methods**
   ```rust
   impl RedisStore {
       pub async fn store_challenge(&self, challenge: &AuthChallenge) -> Result<(), Error> {
           let mut conn = self.client.get_async_connection().await?;
           let key = self.challenge_key(&challenge.challenge_id);
           let value = serde_json::to_string(&challenge)?;
           let expiry = (challenge.expiration - Utc::now())
               .num_seconds()
               .max(0) as usize;
           
           redis::cmd("SET")
               .arg(&key)
               .arg(value)
               .arg("EX")
               .arg(expiry)
               .query_async(&mut conn)
               .await?;
           
           Ok(())
       }
   }
   ```

## Step 4: Authentication Flow Implementation

1. **API Routes (`src/auth/routes.rs`)**
   ```rust
   use actix_web::{web, HttpResponse, Scope};

   pub fn auth_routes() -> Scope {
       web::scope("/auth")
           .service(request_challenge)
           .service(verify_auth)
   }

   #[post("/challenge")]
   async fn request_challenge(
       did: Json<RequestChallenge>,
       state: Data<AppState>,
   ) -> Result<Json<AuthChallenge>, Error> {
       // Implementation
   }
   ```

2. **Middleware Setup (`src/auth/middleware.rs`)**
   ```rust
   pub struct DIDAuthMiddleware {
       resolver: Arc<MultiResolver>,
       store: Arc<RedisStore>,
   }

   impl DIDAuthMiddleware {
       pub fn new(config: DIDAuthConfig) -> Self {
           // Implementation
       }
   }
   ```

## Step 5: Error Handling

1. **Create Error Types (`src/auth/error.rs`)**
   ```rust
   #[derive(Debug, thiserror::Error)]
   pub enum AuthError {
       #[error("Invalid DID format: {0}")]
       InvalidDID(String),

       #[error("Challenge expired")]
       ChallengeExpired,

       #[error("Invalid signature")]
       InvalidSignature,

       #[error("Storage error: {0}")]
       StorageError(#[from] redis::RedisError),
   }
   ```

## Step 6: Configuration

1. **Environment Setup**
   ```bash
   # .env file
   REDIS_URL=redis://localhost:6379
   DID_CHALLENGE_TIMEOUT=300
   DID_RESOLVER_CACHE_TTL=3600
   JWT_SECRET=your-secret-key
   ```

2. **Configuration Structure**
   ```rust
   #[derive(Debug, Clone)]
   pub struct DIDConfig {
       pub redis: RedisConfig,
       pub resolver: ResolverConfig,
       pub challenge: ChallengeConfig,
       pub jwt: JWTConfig,
   }
   ```

## Step 7: Testing Setup

1. **Create Test Utilities**
   ```rust
   #[cfg(test)]
   mod test_utils {
       pub fn create_test_did() -> String {
           "did:example:123".to_string()
       }

       pub fn create_test_challenge() -> AuthChallenge {
           AuthChallenge::new(create_test_did())
       }
   }
   ```

2. **Integration Test Example**
   ```rust
   #[actix_web::test]
   async fn test_full_auth_flow() {
       let app = test::init_service(
           App::new()
               .configure(test_config)
               .service(auth_routes())
       ).await;

       // Test implementation
   }
   ```

## Step 8: Deployment Preparation

1. **Health Check Implementation**
   ```rust
   #[get("/health")]
   async fn health_check(
       state: Data<AppState>,
   ) -> Result<HttpResponse, Error> {
       // Check Redis connection
       state.store.ping().await?;
       
       // Check resolver status
       state.resolver.health_check().await?;
       
       Ok(HttpResponse::Ok().finish())
   }
   ```

2. **Metrics Setup**
   ```rust
   pub struct DIDMetrics {
       pub resolution_histogram: Histogram,
       pub challenge_counter: Counter,
       pub error_counter: Counter,
   }
   ```

## Usage Examples

1. **Basic Authentication Flow**
   ```rust
   // 1. Request challenge
   let challenge_resp = client
       .post("/auth/challenge")
       .json(&json!({ "did": "did:example:123" }))
       .send()
       .await?;

   // 2. Sign challenge (client-side)
   let signature = sign_challenge(&challenge_resp.challenge);

   // 3. Verify authentication
   let auth_resp = client
       .post("/auth/verify")
       .json(&json!({
           "challenge_id": challenge_resp.challenge_id,
           "signature": signature,
       }))
       .send()
       .await?;
   ```

2. **Protected Route Example**
   ```rust
   #[get("/protected")]
   async fn protected_route(
       auth: DIDAuth,
       state: Data<AppState>,
   ) -> Result<HttpResponse, Error> {
       // Access authenticated DID
       let did = auth.did();
       
       Ok(HttpResponse::Ok().json(json!({
           "message": "Protected resource",
           "did": did
       })))
   }
   ```

## Common Issues and Solutions

1. **Challenge Expiration**
   - Ensure system clocks are synchronized
   - Adjust timeout in configuration if needed
   - Handle timezone differences properly

2. **Redis Connection Issues**
   - Implement connection pooling
   - Add retry mechanism
   - Monitor connection health

3. **DID Resolution Failures**
   - Implement fallback resolvers
   - Cache successful resolutions
   - Add timeout handling

## Next Steps

1. **Security Enhancements**
   - Implement rate limiting
   - Add request signing
   - Set up monitoring alerts

2. **Performance Optimization**
   - Fine-tune cache settings
   - Implement batch processing
   - Optimize database queries

3. **Feature Additions**
   - Add support for more DID methods
   - Implement credential verification
   - Add key rotation support 