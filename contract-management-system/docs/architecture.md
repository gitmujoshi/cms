# Contract Management System Architecture

## System Overview

The Contract Management System is a decentralized application that combines traditional database storage with blockchain-based immutable ledger for contract management. The system uses DIDs (Decentralized Identifiers) for authentication and digital signatures.

## Architecture Diagram

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

## Component Details

### 1. Client Layer
- **Web UI**: React-based frontend for user interactions
- **CLI Tool**: Command-line interface for automation and scripting

### 2. API Layer
- **API Gateway**: Main entry point for all client requests
- **Auth Service**: Handles DID-based authentication
- **Rate Limiter**: Prevents abuse and ensures fair usage

### 3. Service Layer
- **Contract Service**: Core business logic for contract management
- **User Service**: User and organization management
- **Audit Service**: Logging and audit trail management
- **Ledger Service**: Blockchain interaction and event recording

### 4. Storage Layer
- **PostgreSQL**: Primary data storage
- **Blockchain**: Immutable ledger for contract events
- **Redis Cache**: Performance optimization

## Data Flow Diagrams

### Contract Creation Flow

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant ContractService
    participant Database
    participant LedgerService
    participant Blockchain

    Client->>API: Create Contract Request
    API->>ContractService: Create Contract
    ContractService->>Database: Store Contract Data
    Database-->>ContractService: Contract Created
    ContractService->>LedgerService: Record Creation Event
    LedgerService->>Blockchain: Store Event
    Blockchain-->>LedgerService: Transaction Hash
    LedgerService-->>ContractService: Event Recorded
    ContractService-->>API: Contract Details
    API-->>Client: Success Response
```

### Contract Signing Flow

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant ContractService
    participant Database
    participant LedgerService
    participant Blockchain

    Client->>API: Sign Contract Request
    API->>ContractService: Verify Signature
    ContractService->>Database: Update Contract
    Database-->>ContractService: Contract Updated
    ContractService->>LedgerService: Record Signature Event
    LedgerService->>Blockchain: Store Event
    Blockchain-->>LedgerService: Transaction Hash
    LedgerService-->>ContractService: Event Recorded
    ContractService-->>API: Updated Contract
    API-->>Client: Success Response
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
}
```

### Event Types
1. **CREATED**: Initial contract creation
2. **SIGNED**: Contract signature
3. **UPDATED**: Contract modifications
4. **VOIDED**: Contract voiding

### State Verification
The system maintains contract state integrity through:
1. Content hashing
2. Blockchain event verification
3. Dual-storage validation

## Security Architecture

```mermaid
graph TB
    subgraph Security Layers
        Auth[Authentication]
        Authz[Authorization]
        Crypto[Cryptography]
        BC[Blockchain]
    end

    subgraph Authentication
        DID[DID Verification]
        JWT[JWT Tokens]
        Challenge[Challenge-Response]
    end

    subgraph Authorization
        RBAC[Role-Based Access]
        Perms[Permissions]
    end

    subgraph Cryptography
        Ed25519[Ed25519 Signatures]
        Hash[Content Hashing]
    end

    subgraph Blockchain
        Events[Event Log]
        Verify[State Verification]
    end

    Auth --> Authz
    Authz --> Crypto
    Crypto --> BC
```

## API Documentation

See `openapi.yaml` for detailed API specifications.

## Error Handling

The system implements a comprehensive error handling strategy:

```rust
pub enum AppError {
    DatabaseError(DbErr),
    AuthError(String),
    ValidationError(String),
    NotFound(String),
    BlockchainError(String),
    ContractStateError(String),
    SignatureError(String),
    InternalError(String),
}
```

## Monitoring and Metrics

### Key Metrics
1. Contract Operations
2. Blockchain Events
3. Signature Verifications
4. API Response Times
5. Error Rates

### Health Checks
1. Database Connectivity
2. Blockchain Node Status
3. Cache Availability
4. API Endpoints

## Deployment Architecture

```mermaid
graph TB
    subgraph Production
        LB[Load Balancer]
        API1[API Server 1]
        API2[API Server 2]
        DB1[(Primary DB)]
        DB2[(Replica DB)]
        Cache1[(Primary Cache)]
        Cache2[(Backup Cache)]
        BC[Blockchain Node]
    end

    LB --> API1
    LB --> API2
    API1 --> DB1
    API2 --> DB1
    DB1 --> DB2
    API1 --> Cache1
    API2 --> Cache1
    Cache1 --> Cache2
    API1 --> BC
    API2 --> BC
```

## Configuration

Environment variables and configuration files are documented in `.env.example` and `config/`.

## Development Setup

See `README.md` for development environment setup instructions. 