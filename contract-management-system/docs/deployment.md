# Deployment Guide

## Overview

This guide details the deployment process for the Contract Management System in both development and production environments.

## Prerequisites

### Development Environment
```bash
# Required software
- Rust 1.75+
- Docker 24.0+
- Docker Compose 2.21+
- PostgreSQL 15+
- Redis 7.0+
- Node.js 20+ (for frontend)
```

### Production Environment
```bash
# Infrastructure requirements
- Kubernetes 1.28+
- PostgreSQL HA cluster
- Redis cluster
- Ethereum node (or provider)
- SSL/TLS certificates
```

## Configuration

### Environment Variables
```bash
# Core settings
APP_ENV=production
APP_PORT=8080
LOG_LEVEL=info

# Database
DATABASE_URL=postgresql://user:password@host:5432/dbname
DATABASE_POOL_SIZE=10

# Redis
REDIS_URL=redis://host:6379
REDIS_POOL_SIZE=20

# Blockchain
BLOCKCHAIN_RPC_URL=https://eth-node:8545
CONTRACT_ADDRESS=0x...
CHAIN_ID=1
GAS_LIMIT=200000
BLOCKCHAIN_SYNC_INTERVAL=60
BLOCKCHAIN_MAX_RETRIES=3
BLOCKCHAIN_BACKUP_ENABLED=true

# Monitoring
PROMETHEUS_ENDPOINT=http://prometheus:9090
GRAFANA_API_KEY=your-grafana-api-key
JAEGER_ENDPOINT=http://jaeger:14250
METRICS_PORT=9091
HEALTH_CHECK_INTERVAL=30

# Security
JWT_SECRET=your-secret-key
JWT_EXPIRY=24h
CORS_ORIGINS=https://your-domain.com
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW=60
```

## Development Deployment

### Local Setup
```bash
# Clone repository
git clone https://github.com/your-org/contract-management-system
cd contract-management-system

# Install dependencies
cargo build

# Set up database
diesel setup
diesel migration run

# Start services
docker-compose up -d

# Run application
cargo run
```

### Testing
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --features integration

# Run blockchain tests
cargo test --features blockchain
```

## Production Deployment

### Kubernetes Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: contract-management-system
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cms
  template:
    metadata:
      labels:
        app: cms
    spec:
      containers:
      - name: cms
        image: your-registry/cms:latest
        ports:
        - containerPort: 8080
        - containerPort: 9091
        envFrom:
        - configMapRef:
            name: cms-config
        - secretRef:
            name: cms-secrets
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: cms-data
          mountPath: /data
      volumes:
      - name: cms-data
        persistentVolumeClaim:
          claimName: cms-data
```

### Database Migration
```bash
# Apply migrations
kubectl exec -it deployment/cms -- diesel migration run

# Verify migration status
kubectl exec -it deployment/cms -- diesel migration list
```

### Smart Contract Deployment
```bash
# Deploy contract
truffle migrate --network production

# Verify contract
truffle run verify ContractLedger --network production
```

## Monitoring Setup

### Prometheus Configuration
```yaml
# prometheus.yaml
scrape_configs:
  - job_name: 'contract-management-system'
    scrape_interval: 15s
    static_configs:
      - targets: ['cms:8080']
```

### Grafana Setup
```bash
# Import dashboards
curl -X POST http://grafana:3000/api/dashboards/import \
  -H "Content-Type: application/json" \
  -d @dashboards/main.json
```

## Security Configuration

### SSL/TLS Setup
```bash
# Generate certificate
certbot certonly --nginx -d your-domain.com

# Configure nginx
server {
    listen 443 ssl;
    server_name your-domain.com;
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    # ... other SSL configurations
}
```

### Firewall Rules
```bash
# Allow necessary ports
ufw allow 80/tcp
ufw allow 443/tcp
ufw allow 8080/tcp
```

## Backup and Recovery

### Database Backup
```bash
# Automated backup script
#!/bin/bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
pg_dump -h $DB_HOST -U $DB_USER $DB_NAME > backup_$TIMESTAMP.sql
```

### Blockchain Data Backup
```bash
# Export contract events
node scripts/export-events.js > events_backup.json

# Verify backup
node scripts/verify-events.js events_backup.json
```

## Scaling

### Horizontal Scaling
```yaml
# Horizontal Pod Autoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: cms-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: contract-management-system
  minReplicas: 3
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
```

### Cache Scaling
```yaml
# Redis cluster configuration
maxmemory 2gb
maxmemory-policy allkeys-lru
cluster-enabled yes
cluster-node-timeout 5000
```

## Troubleshooting

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

### Recovery Procedures

1. **Service Recovery**
```bash
# Restart service
kubectl rollout restart deployment/cms

# Verify health
kubectl get pods -l app=cms
```

2. **Data Recovery**
```bash
# Restore database
psql $DATABASE_URL < backup_file.sql

# Verify data
psql $DATABASE_URL -c "SELECT COUNT(*) FROM contracts;"
```

## Maintenance

### Regular Tasks
1. Database optimization
2. Log rotation
3. Certificate renewal
4. Security updates
5. Performance monitoring

### Update Procedures
```bash
# Update application
kubectl set image deployment/cms cms=your-registry/cms:new-version

# Rollback if needed
kubectl rollout undo deployment/cms
```

### Kubernetes Resources

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: cms-service
spec:
  selector:
    app: cms
  ports:
    - port: 80
      targetPort: 8080
  type: ClusterIP

# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cms-ingress
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  rules:
    - host: your-domain.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: cms-service
                port:
                  number: 80

# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cms-config
data:
  APP_ENV: production
  LOG_LEVEL: info
  METRICS_PORT: "9091"
  HEALTH_CHECK_INTERVAL: "30"

# secrets.yaml
apiVersion: v1
kind: Secret
metadata:
  name: cms-secrets
type: Opaque
data:
  database-url: <base64-encoded-url>
  jwt-secret: <base64-encoded-secret>
  blockchain-key: <base64-encoded-key>

# network-policy.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: cms-network-policy
spec:
  podSelector:
    matchLabels:
      app: cms
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - podSelector:
            matchLabels:
              app: nginx-ingress
      ports:
        - protocol: TCP
          port: 8080
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: postgres
      ports:
        - protocol: TCP
          port: 5432
    - to:
        - podSelector:
            matchLabels:
              app: redis
      ports:
        - protocol: TCP
          port: 6379
    - to:
        - podSelector:
            matchLabels:
              app: ethereum-node
      ports:
        - protocol: TCP
          port: 8545

# volume-claim.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: cms-data
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
``` 