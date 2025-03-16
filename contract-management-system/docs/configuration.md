# Configuration Guide

This document provides detailed information about configuring the Contract Management System (CMS).

## Configuration Files

The system uses YAML configuration files located in the `config/` directory:

- `development.yaml`: Development environment configuration
- `production.yaml`: Production environment configuration
- `test.yaml`: Test environment configuration

## Configuration Sections

### Server Configuration

```yaml
server:
  # Bind address for the server
  host: "0.0.0.0"
  # Port number to listen on
  port: 8080
  # Number of worker threads
  workers: 16
  # TLS configuration (optional)
  tls:
    # Path to TLS certificate
    cert_path: "/etc/cms/tls/cert.pem"
    # Path to TLS private key
    key_path: "/etc/cms/tls/key.pem"
```

### Database Configuration

```yaml
database:
  # PostgreSQL connection URL
  url: "postgres://user:password@host:5432/dbname"
  # Maximum number of connections in the pool
  max_connections: 50
  # Connection idle timeout in seconds
  idle_timeout_seconds: 60
  # SSL mode for database connections
  ssl_mode: "verify-full"
  # Path to SSL certificate authority
  ssl_cert_path: "/etc/cms/db/ca.pem"
```

### Redis Configuration

```yaml
redis:
  # Redis connection URL
  url: "redis://host:6379"
  # Connection pool size
  pool_size: 20
  # Enable TLS for Redis connections
  tls_enabled: true
  # Redis Sentinel configuration (optional)
  sentinel:
    # Master node name
    master_name: "mymaster"
    # List of sentinel nodes
    nodes: ["sentinel-0:26379", "sentinel-1:26379"]
```

### Blockchain Configuration

```yaml
blockchain:
  # Ethereum node URL
  node_url: "https://eth.internal:8545"
  # Chain ID for the network
  chain_id: 1
  # Smart contract address
  contract_address: "${CONTRACT_ADDRESS}"
  # Number of block confirmations required
  confirmations: 12
  # Gas limit for transactions
  gas_limit: 2000000
  # Path to private key file
  private_key_path: "/etc/cms/eth/key.json"
```

### Authentication Configuration

```yaml
auth:
  # Secret for JWT token signing
  jwt_secret: "${JWT_SECRET}"
  # Token expiry time in hours
  token_expiry_hours: 12
  # Challenge timeout in seconds
  challenge_timeout_seconds: 180
  # Rate limiting configuration
  rate_limit:
    # Maximum requests per minute
    requests_per_minute: 60
    # Burst allowance
    burst: 10
```

### Monitoring Configuration

```yaml
monitoring:
  # Path for Prometheus metrics endpoint
  metrics_path: "/metrics"
  # Path for health check endpoint
  health_check_path: "/health"
  # Enable distributed tracing
  tracing_enabled: true
  # Jaeger configuration
  jaeger:
    # Jaeger collector endpoint
    endpoint: "http://jaeger.internal:14268/api/traces"
```

## Environment Variables

The following environment variables can be used to override configuration values:

- `CMS_SERVER_PORT`: Override server port
- `CMS_DB_URL`: Override database URL
- `CMS_REDIS_URL`: Override Redis URL
- `CMS_BLOCKCHAIN_NODE_URL`: Override blockchain node URL
- `CMS_JWT_SECRET`: Override JWT secret
- `CMS_CONTRACT_ADDRESS`: Override smart contract address

## Security Considerations

1. **Secrets Management**
   - Never commit secrets to version control
   - Use environment variables or secure secret management systems
   - Rotate secrets regularly

2. **TLS Configuration**
   - Always use TLS in production
   - Keep certificates up to date
   - Use strong cipher suites

3. **Database Security**
   - Use SSL for database connections
   - Implement proper access controls
   - Regular security audits

## Best Practices

1. **Environment Separation**
   - Use different configurations for each environment
   - Never use development settings in production
   - Document all configuration changes

2. **Monitoring**
   - Enable metrics collection
   - Set up proper logging
   - Configure alerting thresholds

3. **Performance**
   - Tune connection pools based on load
   - Monitor resource usage
   - Implement caching strategies

## Troubleshooting

Common configuration issues and their solutions:

1. **Database Connection Issues**
   ```bash
   # Check database connectivity
   psql $CMS_DB_URL -c "\dt"
   ```

2. **Redis Connection Issues**
   ```bash
   # Test Redis connection
   redis-cli -u $CMS_REDIS_URL ping
   ```

3. **Blockchain Node Issues**
   ```bash
   # Check node synchronization
   curl -X POST -H "Content-Type: application/json" \
     --data '{"jsonrpc":"2.0","method":"eth_syncing","params":[],"id":1}' \
     $CMS_BLOCKCHAIN_NODE_URL
   ``` 