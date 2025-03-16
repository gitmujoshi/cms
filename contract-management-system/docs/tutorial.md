# Contract Management System Tutorial

## Table of Contents
1. [Project Overview](#project-overview)
2. [System Architecture](#system-architecture)
3. [DID Integration](#did-integration)
4. [Blockchain Integration](#blockchain-integration)
5. [Infrastructure Setup](#infrastructure-setup)
6. [Development Guide](#development-guide)
7. [Deployment Guide](#deployment-guide)
8. [Monitoring and Operations](#monitoring-and-operations)
9. [Configuration Examples](#configuration-examples)
10. [Additional Flow Diagrams](#additional-flow-diagrams)
11. [Client Examples](#client-examples)

## Project Overview

The Contract Management System (CMS) is a decentralized application that combines traditional database storage with blockchain-based immutable ledger for managing digital contracts. It uses Decentralized Identifiers (DIDs) for authentication and digital signatures.

### Key Features
- DID-based authentication and signatures
- Blockchain-backed contract verification
- Immutable audit trail
- Scalable microservices architecture
- Comprehensive monitoring and alerting

## System Architecture

### High-Level Architecture
```mermaid
graph TB
    subgraph Client Layer
        UI[Web UI]
        CLI[CLI Tool]
    end

    subgraph API Layer
        API[API Gateway]
        Auth[Auth Service]
        Rate[Rate Limiter]
    end

    subgraph Service Layer
        CS[Contract Service]
        US[User Service]
        AS[Audit Service]
        LS[Ledger Service]
    end

    subgraph Storage Layer
        DB[(PostgreSQL)]
        BC[Blockchain]
        Cache[(Redis Cache)]
    end

    UI --> API
    CLI --> API
    API --> Auth
    API --> Rate
    Auth --> US
    Auth --> CS
    CS --> DB
    CS --> LS
    LS --> BC
    CS --> Cache
    US --> DB
    AS --> DB
    AS --> BC
```

### Component Responsibilities

1. **Client Layer**
   - Web UI: React-based frontend for user interactions
   - CLI: Command-line interface for automation

2. **API Layer**
   - API Gateway: Main entry point for client requests
   - Auth Service: DID-based authentication
   - Rate Limiter: Request throttling

3. **Service Layer**
   - Contract Service: Core business logic
   - User Service: Identity management
   - Audit Service: Logging and tracking
   - Ledger Service: Blockchain interactions

4. **Storage Layer**
   - PostgreSQL: Primary data storage
   - Blockchain: Immutable event log
   - Redis: Performance caching

## DID Integration

### DID Authentication Flow

```mermaid
sequenceDiagram
    participant User
    participant API
    participant Auth
    participant DID Resolver
    
    User->>API: Request Access
    API->>Auth: Generate Challenge
    Auth->>User: Return Challenge
    User->>User: Sign Challenge with DID
    User->>API: Submit Signed Challenge
    API->>Auth: Verify Signature
    Auth->>DID Resolver: Resolve DID Document
    DID Resolver->>Auth: Return DID Document
    Auth->>Auth: Verify Signature
    Auth->>API: Authentication Result
    API->>User: JWT Token
```

### Implementation Details

1. **Challenge Generation**
```rust
pub async fn generate_challenge(did: &str) -> Result<Challenge, Error> {
    let nonce = generate_secure_nonce();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    
    Ok(Challenge {
        nonce,
        timestamp,
        did: did.to_string(),
    })
}
```

2. **Signature Verification**
```rust
pub async fn verify_signature(
    challenge: &Challenge,
    signature: &str,
    did_document: &DIDDocument
) -> Result<bool, Error> {
    let message = create_challenge_message(challenge);
    let verification_method = did_document
        .get_verification_method()?;
    
    verify_ed25519_signature(
        &message,
        signature,
        &verification_method.public_key
    )
}
```

## Blockchain Integration

### Smart Contract Structure

```solidity
contract ContractLedger {
    struct Event {
        string eventType;
        string data;
        uint256 timestamp;
    }

    mapping(string => Event[]) private contractEvents;
    mapping(string => string) private contractStates;

    event ContractEvent(
        string contractId,
        string eventType,
        string data,
        uint256 timestamp
    );

    function recordEvent(
        string memory contractId,
        string memory eventType,
        string memory data
    ) public {
        Event memory newEvent = Event(
            eventType,
            data,
            block.timestamp
        );
        
        contractEvents[contractId].push(newEvent);
        emit ContractEvent(
            contractId,
            eventType,
            data,
            block.timestamp
        );
    }
}
```

### Ledger Service Implementation

```rust
pub struct LedgerService {
    provider: Arc<Provider<Http>>,
    contract: ContractLedger<Provider<Http>>,
    wallet: LocalWallet,
}

impl LedgerService {
    pub async fn record_contract_event(
        &self,
        contract_id: &str,
        event_type: EventType,
        data: &str
    ) -> Result<H256, Error> {
        let tx = self.contract
            .record_event(contract_id, event_type, data)
            .send()
            .await?;
        
        Ok(tx.tx_hash())
    }
}
```

## Infrastructure Setup

### Prerequisites

1. **Development Environment**
```bash
# Install required software
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
brew install docker docker-compose postgresql redis node

# Clone repository
git clone https://github.com/your-org/contract-management-system
cd contract-management-system

# Set up environment
cp .env.example .env
```

2. **Production Environment**
```bash
# Install Kubernetes
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Install Helm
curl https://raw.githubusercontent.com/helm/helm/main/scripts/get-helm-3 | bash
```

### Database Setup

1. **Development**
```bash
# Start PostgreSQL
docker-compose up -d postgres

# Run migrations
diesel setup
diesel migration run
```

2. **Production**
```bash
# Deploy PostgreSQL HA cluster
helm repo add bitnami https://charts.bitnami.com/bitnami
helm install postgres bitnami/postgresql-ha

# Apply schema
kubectl exec -it postgres-0 -- psql -U postgres -f schema.sql
```

### Redis Setup

```bash
# Development
docker-compose up -d redis

# Production
helm install redis bitnami/redis-cluster
```

### Blockchain Node

```bash
# Development (local node)
docker run -d --name ethereum-node \
    -p 8545:8545 \
    ethereum/client-go:latest

# Production
helm install ethereum-node stable/ethereum
```

## Development Guide

### Project Structure
```
contract-management-system/
├── src/
│   ├── api/           # API endpoints
│   ├── auth/          # Authentication
│   ├── blockchain/    # Blockchain integration
│   ├── contracts/     # Smart contracts
│   ├── models/        # Data models
│   └── services/      # Business logic
├── migrations/        # Database migrations
├── tests/            # Test suites
└── docs/             # Documentation
```

### Running the Project

1. **Start Dependencies**
```bash
docker-compose up -d
```

2. **Run Migrations**
```bash
diesel migration run
```

3. **Start Development Server**
```bash
cargo run
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --features integration

# Blockchain tests
cargo test --features blockchain
```

## Deployment Guide

### Kubernetes Deployment

1. **Create Namespace**
```bash
kubectl create namespace cms
```

2. **Deploy Secrets**
```bash
kubectl create secret generic cms-secrets \
    --from-file=.env \
    --namespace cms
```

3. **Deploy Application**
```bash
kubectl apply -f k8s/ --namespace cms
```

### Monitoring Setup

1. **Deploy Prometheus**
```bash
helm install prometheus prometheus-community/prometheus
```

2. **Deploy Grafana**
```bash
helm install grafana grafana/grafana
```

3. **Configure Dashboards**
```bash
kubectl apply -f monitoring/dashboards/
```

## Monitoring and Operations

### Key Metrics

1. **API Metrics**
```prometheus
# Request rates
http_requests_total{method="GET|POST|PUT|DELETE"}

# Response times
http_request_duration_seconds
```

2. **Blockchain Metrics**
```prometheus
# Transaction status
blockchain_transactions_total{status="success|failed"}

# Gas usage
blockchain_gas_used_total
```

### Alerting Rules

```yaml
groups:
- name: critical_alerts
  rules:
  - alert: HighErrorRate
    expr: sum(rate(http_requests_total{status=~"5.."}[5m])) > 0.05
    for: 5m
    labels:
      severity: critical
```

### Backup Procedures

1. **Database Backup**
```bash
#!/bin/bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
pg_dump -h $DB_HOST -U $DB_USER $DB_NAME > backup_$TIMESTAMP.sql
```

2. **Blockchain Data Backup**
```bash
node scripts/export-events.js > events_backup.json
```

### Maintenance

1. **Regular Tasks**
- Database optimization
- Log rotation
- Certificate renewal
- Security updates
- Performance monitoring

2. **Update Procedures**
```bash
# Update application
kubectl set image deployment/cms cms=your-registry/cms:new-version

# Rollback if needed
kubectl rollout undo deployment/cms
```

## Security Considerations

### Authentication
- DID-based authentication
- JWT token expiry
- Rate limiting
- IP filtering

### Data Protection
- Encryption at rest
- TLS in transit
- Key rotation
- Access control

### Blockchain Security
- Gas limits
- Transaction verification
- Smart contract auditing
- Key management

## Best Practices

1. **Development**
- Follow Rust coding standards
- Write comprehensive tests
- Document all APIs
- Use proper error handling

2. **Operations**
- Regular security updates
- Automated deployments
- Continuous monitoring
- Regular backups

3. **Security**
- Regular security audits
- Dependency updates
- Access review
- Incident response planning

## Troubleshooting Guide

### Common Issues

1. **Database Connectivity**
```bash
# Check connection
psql $DATABASE_URL -c "\dt"

# Check logs
kubectl logs deployment/cms -c database
```

2. **Blockchain Sync**
```bash
# Check sync status
curl -X GET http://localhost:8080/blockchain/health

# Check node logs
kubectl logs deployment/ethereum-node
```

3. **Performance Issues**
```bash
# Check metrics
curl -X GET http://localhost:8080/metrics

# Profile application
cargo flamegraph
```

## Additional Resources

1. **Documentation**
- API Documentation
- Architecture Guide
- Security Guide
- Operations Manual

2. **Examples**
- Integration Examples
- Client SDKs
- Sample Applications
- Test Cases 

## Configuration Examples

### Development Configuration

```yaml
# config/development.yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

database:
  url: "postgres://postgres:password@localhost:5432/cms"
  max_connections: 10
  idle_timeout_seconds: 300

redis:
  url: "redis://localhost:6379"
  pool_size: 5

blockchain:
  node_url: "http://localhost:8545"
  chain_id: 1337
  contract_address: "0x..."
  confirmations: 1
  gas_limit: 2000000

auth:
  jwt_secret: "development-secret"
  token_expiry_hours: 24
  challenge_timeout_seconds: 300

monitoring:
  metrics_path: "/metrics"
  health_check_path: "/health"
  tracing_enabled: true
```

### Production Configuration

```yaml
# config/production.yaml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 16
  tls:
    cert_path: "/etc/cms/tls/cert.pem"
    key_path: "/etc/cms/tls/key.pem"

database:
  url: "postgres://cms:${DB_PASSWORD}@cms-db.internal:5432/cms"
  max_connections: 50
  idle_timeout_seconds: 60
  ssl_mode: "verify-full"
  ssl_cert_path: "/etc/cms/db/ca.pem"

redis:
  url: "redis://cms-redis.internal:6379"
  pool_size: 20
  tls_enabled: true
  sentinel:
    master_name: "mymaster"
    nodes: ["sentinel-0:26379", "sentinel-1:26379"]

blockchain:
  node_url: "https://eth.internal:8545"
  chain_id: 1
  contract_address: "${CONTRACT_ADDRESS}"
  confirmations: 12
  gas_limit: 2000000
  private_key_path: "/etc/cms/eth/key.json"

auth:
  jwt_secret: "${JWT_SECRET}"
  token_expiry_hours: 12
  challenge_timeout_seconds: 180
  rate_limit:
    requests_per_minute: 60
    burst: 10

monitoring:
  metrics_path: "/metrics"
  health_check_path: "/health"
  tracing_enabled: true
  jaeger:
    endpoint: "http://jaeger.internal:14268/api/traces"
```

## Additional Flow Diagrams

### Contract Creation and Signing Flow

```mermaid
sequenceDiagram
    participant Creator
    participant API
    participant ContractService
    participant BlockchainService
    participant Database
    participant Blockchain

    Creator->>API: Create Contract Request
    API->>ContractService: Validate Contract Data
    ContractService->>Database: Store Draft Contract
    Database-->>ContractService: Contract ID
    ContractService->>BlockchainService: Record Creation Event
    BlockchainService->>Blockchain: Submit Transaction
    Blockchain-->>BlockchainService: Transaction Hash
    BlockchainService->>Database: Update Contract Status
    ContractService-->>API: Contract Details
    API-->>Creator: Contract Created Response
```

### Authentication and Authorization Flow

```mermaid
sequenceDiagram
    participant User
    participant API
    participant AuthService
    participant DIDResolver
    participant Redis
    participant JWT

    User->>API: Request Access
    API->>AuthService: Generate Challenge
    AuthService->>Redis: Store Challenge
    Redis-->>AuthService: Challenge Stored
    AuthService-->>API: Challenge
    API-->>User: Challenge Response
    User->>API: Submit Signed Challenge
    API->>AuthService: Verify Challenge
    AuthService->>Redis: Retrieve Challenge
    Redis-->>AuthService: Challenge Data
    AuthService->>DIDResolver: Resolve DID
    DIDResolver-->>AuthService: DID Document
    AuthService->>AuthService: Verify Signature
    AuthService->>JWT: Generate Token
    JWT-->>AuthService: JWT Token
    AuthService-->>API: Auth Result
    API-->>User: JWT Token Response
```

### Contract State Verification Flow

```mermaid
sequenceDiagram
    participant Verifier
    participant API
    participant ContractService
    participant BlockchainService
    participant Database
    participant Blockchain

    Verifier->>API: Verify Contract Request
    API->>ContractService: Get Contract Details
    ContractService->>Database: Fetch Contract
    Database-->>ContractService: Contract Data
    ContractService->>BlockchainService: Get Contract Events
    BlockchainService->>Blockchain: Query Events
    Blockchain-->>BlockchainService: Event Log
    BlockchainService->>BlockchainService: Verify State
    BlockchainService-->>ContractService: Verification Result
    ContractService-->>API: Verification Details
    API-->>Verifier: Verification Response
```

## Client Examples

### Python Client

```python
from dataclasses import dataclass
import httpx
from typing import Optional
import jwt
from eth_account import Account
from eth_account.messages import encode_defunct

@dataclass
class ContractClient:
    base_url: str
    jwt_token: Optional[str] = None
    
    async def register_did(self, did: str, public_key: str) -> dict:
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.base_url}/api/v1/auth/register",
                json={"did": did, "public_key": public_key}
            )
            return response.json()
    
    async def authenticate(self, did: str, private_key: str) -> None:
        # Get challenge
        async with httpx.AsyncClient() as client:
            challenge_resp = await client.post(
                f"{self.base_url}/api/v1/auth/challenge",
                json={"did": did}
            )
            challenge = challenge_resp.json()["challenge"]
            
            # Sign challenge
            account = Account.from_key(private_key)
            message = encode_defunct(text=challenge)
            signature = account.sign_message(message)
            
            # Verify signature
            verify_resp = await client.post(
                f"{self.base_url}/api/v1/auth/verify",
                json={
                    "did": did,
                    "challenge": challenge,
                    "signature": signature.signature.hex()
                }
            )
            self.jwt_token = verify_resp.json()["token"]
    
    async def create_contract(self, contract_data: dict) -> dict:
        if not self.jwt_token:
            raise ValueError("Not authenticated")
            
        async with httpx.AsyncClient() as client:
            response = await client.post(
                f"{self.base_url}/api/v1/contracts",
                json=contract_data,
                headers={"Authorization": f"Bearer {self.jwt_token}"}
            )
            return response.json()
    
    async def get_contract(self, contract_id: str) -> dict:
        if not self.jwt_token:
            raise ValueError("Not authenticated")
            
        async with httpx.AsyncClient() as client:
            response = await client.get(
                f"{self.base_url}/api/v1/contracts/{contract_id}",
                headers={"Authorization": f"Bearer {self.jwt_token}"}
            )
            return response.json()
```

### JavaScript/TypeScript Client

```typescript
import axios, { AxiosInstance } from 'axios';
import { ethers } from 'ethers';

interface ContractData {
  title: string;
  description: string;
  consumer_did: string;
  terms: string;
  valid_from: string;
  valid_until: string;
}

class ContractClient {
  private baseUrl: string;
  private client: AxiosInstance;
  private jwtToken?: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
    this.client = axios.create({
      baseURL: this.baseUrl,
    });
  }

  async registerDid(did: string, publicKey: string): Promise<any> {
    const response = await this.client.post('/api/v1/auth/register', {
      did,
      public_key: publicKey,
    });
    return response.data;
  }

  async authenticate(did: string, privateKey: string): Promise<void> {
    // Get challenge
    const challengeResp = await this.client.post('/api/v1/auth/challenge', {
      did,
    });
    const challenge = challengeResp.data.challenge;

    // Sign challenge
    const wallet = new ethers.Wallet(privateKey);
    const signature = await wallet.signMessage(challenge);

    // Verify signature
    const verifyResp = await this.client.post('/api/v1/auth/verify', {
      did,
      challenge,
      signature,
    });
    this.jwtToken = verifyResp.data.token;

    // Update client headers
    this.client.defaults.headers.common['Authorization'] = `Bearer ${this.jwtToken}`;
  }

  async createContract(contractData: ContractData): Promise<any> {
    if (!this.jwtToken) {
      throw new Error('Not authenticated');
    }

    const response = await this.client.post('/api/v1/contracts', contractData);
    return response.data;
  }

  async getContract(contractId: string): Promise<any> {
    if (!this.jwtToken) {
      throw new Error('Not authenticated');
    }

    const response = await this.client.get(`/api/v1/contracts/${contractId}`);
    return response.data;
  }

  async getContractEvents(contractId: string): Promise<any> {
    if (!this.jwtToken) {
      throw new Error('Not authenticated');
    }

    const response = await this.client.get(
      `/api/v1/contracts/${contractId}/events`
    );
    return response.data;
  }

  async verifyContract(contractId: string): Promise<any> {
    if (!this.jwtToken) {
      throw new Error('Not authenticated');
    }

    const response = await this.client.post(
      `/api/v1/contracts/${contractId}/verify`
    );
    return response.data;
  }
}

export default ContractClient;
```

### Rust Client

```rust
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use ethers::{
    signers::{LocalWallet, Signer},
    types::Signature,
};
use anyhow::Result;

#[derive(Debug, Serialize)]
struct ContractData {
    title: String,
    description: String,
    consumer_did: String,
    terms: String,
    valid_from: String,
    valid_until: String,
}

#[derive(Debug, Deserialize)]
struct AuthResponse {
    token: String,
}

pub struct ContractClient {
    base_url: String,
    client: Client,
    jwt_token: Option<String>,
}

impl ContractClient {
    pub fn new(base_url: &str) -> Result<Self> {
        Ok(Self {
            base_url: base_url.to_string(),
            client: Client::new(),
            jwt_token: None,
        })
    }

    pub async fn register_did(&self, did: &str, public_key: &str) -> Result<()> {
        self.client
            .post(&format!("{}/api/v1/auth/register", self.base_url))
            .json(&serde_json::json!({
                "did": did,
                "public_key": public_key,
            }))
            .send()
            .await?;
        Ok(())
    }

    pub async fn authenticate(&mut self, did: &str, wallet: &LocalWallet) -> Result<()> {
        // Get challenge
        let challenge_resp = self.client
            .post(&format!("{}/api/v1/auth/challenge", self.base_url))
            .json(&serde_json::json!({ "did": did }))
            .send()
            .await?;
        
        let challenge: String = challenge_resp.json::<serde_json::Value>().await?
            .get("challenge")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        // Sign challenge
        let signature: Signature = wallet.sign_message(challenge.as_bytes()).await?;

        // Verify signature
        let verify_resp = self.client
            .post(&format!("{}/api/v1/auth/verify", self.base_url))
            .json(&serde_json::json!({
                "did": did,
                "challenge": challenge,
                "signature": format!("0x{}", hex::encode(signature.to_vec())),
            }))
            .send()
            .await?;

        let auth_response: AuthResponse = verify_resp.json().await?;
        self.jwt_token = Some(auth_response.token);
        Ok(())
    }

    pub async fn create_contract(&self, contract_data: ContractData) -> Result<serde_json::Value> {
        let token = self.jwt_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );

        let response = self.client
            .post(&format!("{}/api/v1/contracts", self.base_url))
            .headers(headers)
            .json(&contract_data)
            .send()
            .await?;

        Ok(response.json().await?)
    }

    pub async fn get_contract(&self, contract_id: &str) -> Result<serde_json::Value> {
        let token = self.jwt_token.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", token))?,
        );

        let response = self.client
            .get(&format!("{}/api/v1/contracts/{}", self.base_url, contract_id))
            .headers(headers)
            .send()
            .await?;

        Ok(response.json().await?)
    }
} 