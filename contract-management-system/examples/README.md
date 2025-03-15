# Contract Management System Client Examples

This directory contains example code demonstrating how to interact with the Contract Management System API using DIDs for authentication and contract signing.

## Overview

The example client code shows how to:
1. Generate and manage DID keypairs
2. Authenticate using DID-based challenge-response
3. Create and manage contracts
4. Sign contracts using DIDs
5. Verify contract signatures

## Prerequisites

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1.0", features = ["serde", "v4"] }
ed25519-dalek = "1.0"
bs58 = "0.4"
rand = "0.8"
```

## Usage

### Basic Usage

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let mut client = ContractClient::new("http://localhost:8080".to_string());

    // Authenticate
    client.authenticate().await?;

    // Create a contract
    let contract = client.create_contract(
        "Test Contract",
        "A test contract between two parties",
        "did:example:consumer123",
        "Contract terms...",
    ).await?;

    // Sign the contract
    let signed_contract = client.sign_contract(contract.id).await?;

    // Verify signatures
    let valid = client.verify_signatures(contract.id).await?;
    println!("Signatures valid: {}", valid);

    Ok(())
}
```

### Complete Contract Workflow

The example includes a complete workflow test that demonstrates:
1. Provider and consumer authentication
2. Contract creation by provider
3. Contract signing by both parties
4. Signature verification

```rust
async fn test_contract_workflow() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize provider and consumer clients
    let mut provider = ContractClient::new("http://localhost:8080".to_string());
    let mut consumer = ContractClient::new("http://localhost:8080".to_string());
    
    provider.authenticate().await?;
    consumer.authenticate().await?;

    // Create and sign contract
    let contract = provider.create_contract(
        "Service Agreement",
        "Agreement for providing consulting services",
        &consumer.keypair.did,
        "Terms and conditions...",
    ).await?;

    // Both parties sign
    let contract = provider.sign_contract(contract.id).await?;
    let contract = consumer.sign_contract(contract.id).await?;

    // Verify signatures
    let valid = provider.verify_signatures(contract.id).await?;
    assert!(valid);

    Ok(())
}
```

## Key Components

### DIDKeyPair

Manages Ed25519 keypairs for DID authentication and signing:
- Generates new keypairs
- Creates DID identifiers
- Signs messages

### ContractClient

Main client for interacting with the API:
- Handles authentication
- Creates and manages contracts
- Signs contracts
- Verifies signatures

## Security Considerations

1. **Key Management**: The example uses in-memory key storage. In production:
   - Use secure key storage (HSM, secure enclave)
   - Implement key rotation
   - Handle key backup and recovery

2. **Authentication**: The example demonstrates basic challenge-response. Consider:
   - Adding request signing
   - Implementing refresh tokens
   - Adding rate limiting

3. **Contract Signing**: The example uses basic Ed25519 signatures. Consider:
   - Supporting multiple signature types
   - Implementing signature timestamping
   - Adding signature expiration

## Running the Examples

1. Start the Contract Management System server:
   ```bash
   cargo run --bin server
   ```

2. Run the example client:
   ```bash
   cargo run --example client
   ```

3. Run the tests:
   ```bash
   cargo test --example client
   ```

## API Documentation

For complete API documentation, see the OpenAPI specification in `docs/api/openapi.yaml`. 