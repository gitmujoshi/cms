use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tokio::sync::RwLock;
use chrono::{DateTime, Duration, Utc};

use crate::auth::did::{DIDError, Result, VerificationMethod};

#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub controller: Vec<String>,
    pub verification_method: Vec<VerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub updated: DateTime<Utc>,
}

#[async_trait]
pub trait DIDResolver: Send + Sync {
    async fn resolve(&self, did: &str) -> Result<Document>;
}

pub struct MultiResolver {
    resolvers: HashMap<String, Arc<dyn DIDResolver>>,
    cache: Arc<RwLock<HashMap<String, (Document, DateTime<Utc>)>>>,
    cache_ttl: Duration,
}

impl MultiResolver {
    pub fn new(cache_ttl: Duration) -> Self {
        Self {
            resolvers: HashMap::new(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl,
        }
    }

    pub fn register_resolver(&mut self, method: String, resolver: Arc<dyn DIDResolver>) {
        self.resolvers.insert(method, resolver);
    }

    pub async fn resolve(&self, did: &str) -> Result<Document> {
        // Check cache first
        if let Some((doc, timestamp)) = self.cache.read().await.get(did) {
            if Utc::now() - *timestamp < self.cache_ttl {
                return Ok(doc.clone());
            }
        }

        // Parse DID method
        let method = parse_did_method(did)
            .ok_or_else(|| DIDError::InvalidFormat(did.to_string()))?;

        // Get resolver for method
        let resolver = self.resolvers
            .get(&method)
            .ok_or_else(|| DIDError::ResolutionFailed(format!("No resolver for method: {}", method)))?;

        // Resolve DID
        let document = resolver.resolve(did).await?;

        // Update cache
        self.cache.write().await.insert(
            did.to_string(),
            (document.clone(), Utc::now()),
        );

        Ok(document)
    }

    pub async fn clear_cache(&self) {
        self.cache.write().await.clear();
    }
}

fn parse_did_method(did: &str) -> Option<String> {
    let parts: Vec<&str> = did.split(':').collect();
    if parts.len() >= 2 && parts[0] == "did" {
        Some(parts[1].to_string())
    } else {
        None
    }
}

// Example implementation for Ethereum DIDs
pub struct EthereumDIDResolver {
    rpc_endpoint: String,
}

#[async_trait]
impl DIDResolver for EthereumDIDResolver {
    async fn resolve(&self, did: &str) -> Result<Document> {
        // Implementation would interact with Ethereum network
        // This is a placeholder that creates a basic document
        Ok(Document {
            id: did.to_string(),
            controller: vec![did.to_string()],
            verification_method: vec![VerificationMethod {
                id: format!("{}#keys-1", did),
                type_: "EcdsaSecp256k1VerificationKey2019".to_string(),
                controller: did.to_string(),
                public_key_base58: "placeholder".to_string(),
            }],
            authentication: vec![format!("{}#keys-1", did)],
            assertion_method: vec![format!("{}#keys-1", did)],
            updated: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockResolver;

    #[async_trait]
    impl DIDResolver for MockResolver {
        async fn resolve(&self, did: &str) -> Result<Document> {
            Ok(Document {
                id: did.to_string(),
                controller: vec![did.to_string()],
                verification_method: vec![],
                authentication: vec![],
                assertion_method: vec![],
                updated: Utc::now(),
            })
        }
    }

    #[tokio::test]
    async fn test_multi_resolver() {
        let mut resolver = MultiResolver::new(Duration::minutes(5));
        resolver.register_resolver(
            "example".to_string(),
            Arc::new(MockResolver) as Arc<dyn DIDResolver>
        );

        let result = resolver.resolve("did:example:123").await;
        assert!(result.is_ok());

        let result = resolver.resolve("did:unknown:123").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_did_method() {
        assert_eq!(parse_did_method("did:example:123"), Some("example".to_string()));
        assert_eq!(parse_did_method("invalid"), None);
    }
} 