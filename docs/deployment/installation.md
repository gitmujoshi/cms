# Contract Management System Deployment Guide

## Prerequisites

### System Requirements
- Linux-based operating system (Ubuntu 20.04 LTS recommended)
- Docker 20.10+
- Docker Compose 2.0+
- PostgreSQL 13+
- Redis 6.0+
- Node.js 16+
- Rust 1.65+

### Hardware Requirements
- CPU: 4+ cores
- RAM: 16GB+
- Storage: 100GB+ SSD
- Network: 1Gbps+

## Installation Steps

### 1. Environment Setup

```bash
# Create deployment directory
mkdir -p /opt/contract-management
cd /opt/contract-management

# Clone repository
git clone https://github.com/your-org/contract-management-system.git
cd contract-management-system
```

### 2. Configuration

Create `.env` file:
```bash
# Database Configuration
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_DB=contract_management
POSTGRES_USER=contract_user
POSTGRES_PASSWORD=your_secure_password

# Redis Configuration
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=your_secure_password

# API Configuration
API_HOST=0.0.0.0
API_PORT=8080
API_SECRET_KEY=your_secure_secret

# Blockchain Configuration
BLOCKCHAIN_NETWORK=mainnet
BLOCKCHAIN_RPC_URL=https://your-rpc-url
BLOCKCHAIN_PRIVATE_KEY=your_private_key

# Security Configuration
JWT_SECRET=your_jwt_secret
ENCRYPTION_KEY=your_encryption_key
```

### 3. Database Setup

```bash
# Create PostgreSQL database
sudo -u postgres psql
CREATE DATABASE contract_management;
CREATE USER contract_user WITH PASSWORD 'your_secure_password';
GRANT ALL PRIVILEGES ON DATABASE contract_management TO contract_user;

# Run migrations
cargo run --bin migrate
```

### 4. Docker Setup

Create `docker-compose.yml`:
```yaml
version: '3.8'

services:
  api:
    build: .
    ports:
      - "8080:8080"
    environment:
      - POSTGRES_HOST=db
      - REDIS_HOST=redis
    depends_on:
      - db
      - redis

  db:
    image: postgres:13
    environment:
      - POSTGRES_DB=contract_management
      - POSTGRES_USER=contract_user
      - POSTGRES_PASSWORD=your_secure_password
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:6
    command: redis-server --requirepass your_secure_password
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### 5. Build and Start Services

```bash
# Build the application
cargo build --release

# Start services
docker-compose up -d

# Verify services
docker-compose ps
```

## Configuration

### 1. API Configuration

```yaml
# config/api.yaml
server:
  host: 0.0.0.0
  port: 8080
  workers: 4
  timeout: 30

security:
  jwt_secret: your_jwt_secret
  encryption_key: your_encryption_key
  rate_limit: 100
  rate_limit_period: 60

logging:
  level: info
  format: json
  output: /var/log/contract-management/api.log
```

### 2. Database Configuration

```yaml
# config/database.yaml
postgres:
  host: localhost
  port: 5432
  database: contract_management
  user: contract_user
  password: your_secure_password
  pool_size: 10
  timeout: 30

redis:
  host: localhost
  port: 6379
  password: your_secure_password
  pool_size: 10
  timeout: 30
```

### 3. Blockchain Configuration

```yaml
# config/blockchain.yaml
network: mainnet
rpc_url: https://your-rpc-url
private_key: your_private_key
gas_limit: 300000
gas_price: 20
```

## Monitoring Setup

### 1. Prometheus Configuration

```yaml
# config/prometheus.yaml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'contract_management'
    static_configs:
      - targets: ['localhost:8080']
```

### 2. Grafana Dashboard

Import the following dashboard configuration:
```json
{
  "dashboard": {
    "title": "Contract Management System",
    "panels": [
      {
        "title": "API Requests",
        "type": "graph",
        "datasource": "Prometheus",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])"
          }
        ]
      }
    ]
  }
}
```

## Security Configuration

### 1. SSL/TLS Setup

```bash
# Generate SSL certificates
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/contract-management.key \
  -out /etc/ssl/certs/contract-management.crt
```

### 2. Firewall Configuration

```bash
# Configure UFW
sudo ufw allow 8080/tcp
sudo ufw allow 5432/tcp
sudo ufw allow 6379/tcp
sudo ufw enable
```

### 3. Security Headers

Add to your API configuration:
```yaml
security:
  headers:
    - name: X-Content-Type-Options
      value: nosniff
    - name: X-Frame-Options
      value: DENY
    - name: X-XSS-Protection
      value: 1; mode=block
    - name: Strict-Transport-Security
      value: max-age=31536000; includeSubDomains
```

## Backup and Recovery

### 1. Database Backup

```bash
# Create backup script
#!/bin/bash
BACKUP_DIR="/var/backups/contract-management"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

pg_dump -U contract_user -d contract_management > $BACKUP_DIR/db_$TIMESTAMP.sql
```

### 2. Configuration Backup

```bash
# Backup configuration files
tar -czf /var/backups/contract-management/config_$TIMESTAMP.tar.gz /etc/contract-management/
```

### 3. Recovery Procedure

```bash
# Database recovery
psql -U contract_user -d contract_management < backup_file.sql

# Configuration recovery
tar -xzf config_backup.tar.gz -C /
```

## Scaling

### 1. Horizontal Scaling

```yaml
# docker-compose.scale.yml
services:
  api:
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '1'
          memory: 2G
```

### 2. Load Balancer Configuration

```nginx
# nginx.conf
upstream contract_management {
    server api1:8080;
    server api2:8080;
    server api3:8080;
}

server {
    listen 80;
    server_name contract-management.example.com;

    location / {
        proxy_pass http://contract_management;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Maintenance

### 1. Regular Maintenance Tasks

```bash
# Database maintenance
vacuumdb -U contract_user -d contract_management

# Log rotation
logrotate /etc/logrotate.d/contract-management
```

### 2. Update Procedure

```bash
# Update application
git pull
cargo build --release
docker-compose down
docker-compose up -d
```

### 3. Monitoring and Alerts

```yaml
# alertmanager.yml
receivers:
  - name: 'email'
    email_configs:
      - to: 'admin@example.com'
        from: 'alerts@contract-management.com'
        smarthost: 'smtp.example.com:587'
        auth_username: 'alert_user'
        auth_password: 'your_password'
```

## Troubleshooting

### Common Issues

1. **Database Connection Issues**
   - Check PostgreSQL service status
   - Verify connection parameters
   - Check firewall settings

2. **API Performance Issues**
   - Monitor resource usage
   - Check for slow queries
   - Verify cache configuration

3. **Authentication Problems**
   - Verify JWT configuration
   - Check token expiration
   - Validate user permissions

### Log Files

- API logs: `/var/log/contract-management/api.log`
- Database logs: `/var/log/postgresql/postgresql-13-main.log`
- System logs: `/var/log/syslog`

## Support

For support, contact:
- Email: support@contract-management.com
- Documentation: https://docs.contract-management.com
- Issue Tracker: https://github.com/your-org/contract-management-system/issues 