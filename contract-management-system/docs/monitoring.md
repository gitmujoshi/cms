# Monitoring and Metrics Specification

## Overview

This document details the monitoring and metrics collection strategy for the Contract Management System.

## Key Metrics

### API Metrics
```prometheus
# Request rates
http_requests_total{method="GET|POST|PUT|DELETE", endpoint="/api/v1/*", status="2xx|4xx|5xx"}

# Response times
http_request_duration_seconds{method, endpoint}

# Active sessions
active_sessions_total
```

### Contract Metrics
```prometheus
# Contract operations
contract_operations_total{operation="create|sign|update|void"}

# Contract states
contracts_by_status{status="draft|pending_signature|active|suspended|terminated"}

# Signature verification time
signature_verification_duration_seconds
```

### Blockchain Metrics
```prometheus
# Transaction status
blockchain_transactions_total{status="success|failed"}

# Gas usage
blockchain_gas_used_total

# Event recording latency
blockchain_event_recording_latency_seconds

# Node sync status
blockchain_node_sync_status{node_id}
```

### Cache Metrics
```prometheus
# Cache hit rate
cache_hit_ratio{cache_type="redis"}

# Cache operation latency
cache_operation_duration_seconds{operation="get|set"}

# Cache memory usage
cache_memory_usage_bytes
```

## Alerting Rules

### Critical Alerts

```yaml
groups:
- name: critical_alerts
  rules:
  - alert: HighErrorRate
    expr: sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) > 0.05
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: High error rate detected

  - alert: BlockchainNodeDown
    expr: blockchain_node_sync_status == 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: Blockchain node is down

  - alert: ContractVerificationFailure
    expr: rate(signature_verification_failures_total[5m]) > 0
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: Contract verification failures detected
```

### Warning Alerts

```yaml
groups:
- name: warning_alerts
  rules:
  - alert: HighLatency
    expr: histogram_quantile(0.95, http_request_duration_seconds) > 2
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: High API latency detected

  - alert: CachePerformanceDegraded
    expr: cache_hit_ratio < 0.8
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: Cache performance is degraded
```

## Dashboards

### Main Dashboard

```grafana
Dashboard {
  uid: "contract-management"
  title: "Contract Management System"
  
  panels: [
    Panel {
      title: "API Request Rate"
      type: "graph"
      metrics: [
        "rate(http_requests_total[5m])"
      ]
    },
    Panel {
      title: "Contract Operations"
      type: "graph"
      metrics: [
        "sum by (operation) (rate(contract_operations_total[5m]))"
      ]
    },
    Panel {
      title: "Blockchain Health"
      type: "stat"
      metrics: [
        "blockchain_node_sync_status"
      ]
    }
  ]
}
```

### Blockchain Dashboard

```grafana
Dashboard {
  uid: "blockchain-metrics"
  title: "Blockchain Operations"
  
  panels: [
    Panel {
      title: "Transaction Success Rate"
      type: "gauge"
      metrics: [
        "sum(blockchain_transactions_total{status='success'}) / sum(blockchain_transactions_total)"
      ]
    },
    Panel {
      title: "Gas Usage Trend"
      type: "graph"
      metrics: [
        "rate(blockchain_gas_used_total[1h])"
      ]
    }
  ]
}
```

## Health Checks

### API Health
```http
GET /health
Response 200:
{
  "status": "healthy",
  "components": {
    "api": "up",
    "database": "up",
    "cache": "up",
    "blockchain": "up"
  }
}
```

### Blockchain Health
```http
GET /blockchain/health
Response 200:
{
  "status": "healthy",
  "node_info": {
    "block_height": 12345678,
    "sync_status": "synced",
    "peer_count": 5
  }
}
```

## Logging

### Log Levels
- ERROR: System errors requiring immediate attention
- WARN: Potential issues that need investigation
- INFO: Normal operation events
- DEBUG: Detailed information for debugging

### Log Format
```json
{
  "timestamp": "2024-03-21T10:15:30Z",
  "level": "INFO",
  "service": "contract-service",
  "trace_id": "abc123",
  "message": "Contract signed",
  "data": {
    "contract_id": "uuid",
    "signer": "did:example:123",
    "status": "success"
  }
}
```

## Tracing

### OpenTelemetry Configuration
```yaml
service_name: contract-management-system
sampler:
  type: probabilistic
  rate: 0.1

exporters:
  jaeger:
    endpoint: http://jaeger:14250
    
processors:
  batch:
    timeout: 1s
    send_batch_size: 100
```

## Incident Response

### Severity Levels
1. **Critical**: System unavailable or data integrity compromised
2. **High**: Major feature unavailable or significant performance degradation
3. **Medium**: Minor feature issues or moderate performance impact
4. **Low**: Cosmetic issues or minimal impact

### Response Procedures
1. Incident detection via monitoring alerts
2. Initial assessment and severity classification
3. Team notification based on severity
4. Investigation and root cause analysis
5. Resolution and recovery
6. Post-incident review and documentation 

# Monitoring Setup

## Prometheus Rules

### Recording Rules
```yaml
groups:
- name: cms_recording_rules
  rules:
  - record: cms:contract_operations:rate5m
    expr: sum by (operation) (rate(contract_operations_total[5m]))
  
  - record: cms:blockchain_transactions:success_rate
    expr: sum(blockchain_transactions_total{status="success"}) / sum(blockchain_transactions_total)
  
  - record: cms:api_latency:p95
    expr: histogram_quantile(0.95, sum by (le) (rate(http_request_duration_seconds_bucket[5m])))
```

## Grafana Dashboard Provisioning

### Dashboard Configuration
```yaml
# /etc/grafana/provisioning/dashboards/cms.yaml
apiVersion: 1
providers:
- name: 'Contract Management System'
  orgId: 1
  folder: 'CMS'
  type: file
  disableDeletion: false
  editable: true
  options:
    path: /var/lib/grafana/dashboards
```

### Data Source Configuration
```yaml
# /etc/grafana/provisioning/datasources/prometheus.yaml
apiVersion: 1
datasources:
- name: Prometheus
  type: prometheus
  access: proxy
  url: http://prometheus:9090
  isDefault: true
  version: 1
  editable: false
```

## Jaeger Configuration

### Agent Configuration
```yaml
# jaeger-agent.yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: jaeger-agent
spec:
  selector:
    matchLabels:
      app: jaeger-agent
  template:
    metadata:
      labels:
        app: jaeger-agent
    spec:
      containers:
      - name: jaeger-agent
        image: jaegertracing/jaeger-agent:latest
        args:
        - --reporter.grpc.host-port=jaeger-collector:14250
        ports:
        - containerPort: 6831
          protocol: UDP
```

### Collector Configuration
```yaml
# jaeger-collector.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: jaeger-collector
spec:
  selector:
    matchLabels:
      app: jaeger-collector
  template:
    metadata:
      labels:
        app: jaeger-collector
    spec:
      containers:
      - name: jaeger-collector
        image: jaegertracing/jaeger-collector:latest
        ports:
        - containerPort: 14250
```

## Alert Manager Configuration

### Alert Routes
```yaml
# alertmanager.yml
route:
  group_by: ['alertname', 'cluster', 'service']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'team-cms'
  routes:
  - match:
      severity: critical
    receiver: 'team-cms-critical'
    repeat_interval: 1h

receivers:
- name: 'team-cms'
  slack_configs:
  - channel: '#cms-alerts'
    send_resolved: true
    
- name: 'team-cms-critical'
  slack_configs:
  - channel: '#cms-critical'
    send_resolved: true
  pagerduty_configs:
  - service_key: '<your-pagerduty-key>'
```

## Metrics Endpoints

### Application Metrics
```rust
#[get("/metrics")]
async fn metrics() -> impl Responder {
    let metrics = prometheus::gather();
    let encoder = TextEncoder::new();
    let mut buffer = vec![];
    encoder.encode(&metrics, &mut buffer).unwrap();
    HttpResponse::Ok()
        .content_type("text/plain")
        .body(buffer)
}
```

### Custom Metrics
```rust
lazy_static! {
    static ref CONTRACT_OPERATIONS: CounterVec = register_counter_vec!(
        "contract_operations_total",
        "Total number of contract operations",
        &["operation"]
    ).unwrap();
    
    static ref SIGNATURE_VERIFICATION_TIME: Histogram = register_histogram!(
        "signature_verification_duration_seconds",
        "Time spent verifying signatures"
    ).unwrap();
}
```

## Dashboard Templates

### Contract Operations Dashboard
```json
{
  "dashboard": {
    "id": null,
    "title": "Contract Operations",
    "tags": ["cms", "contracts"],
    "timezone": "browser",
    "panels": [
      {
        "title": "Contract Creation Rate",
        "type": "graph",
        "datasource": "Prometheus",
        "targets": [
          {
            "expr": "rate(contract_operations_total{operation=\"create\"}[5m])",
            "legendFormat": "Creates/sec"
          }
        ]
      },
      {
        "title": "Signature Success Rate",
        "type": "gauge",
        "datasource": "Prometheus",
        "targets": [
          {
            "expr": "sum(rate(signature_verification_success_total[5m])) / sum(rate(signature_verification_total[5m]))",
            "legendFormat": "Success Rate"
          }
        ]
      }
    ]
  }
}
```

### Blockchain Operations Dashboard
```json
{
  "dashboard": {
    "id": null,
    "title": "Blockchain Operations",
    "tags": ["cms", "blockchain"],
    "timezone": "browser",
    "panels": [
      {
        "title": "Transaction Success Rate",
        "type": "gauge",
        "datasource": "Prometheus",
        "targets": [
          {
            "expr": "cms:blockchain_transactions:success_rate",
            "legendFormat": "Success Rate"
          }
        ]
      },
      {
        "title": "Gas Usage",
        "type": "graph",
        "datasource": "Prometheus",
        "targets": [
          {
            "expr": "rate(blockchain_gas_used_total[1h])",
            "legendFormat": "Gas/hour"
          }
        ]
      }
    ]
  }
}
```

## Log Aggregation

### Fluentd Configuration
```yaml
# fluentd-configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: fluentd-config
data:
  fluent.conf: |
    <source>
      @type tail
      path /var/log/containers/cms-*.log
      pos_file /var/log/cms.log.pos
      tag cms.*
      read_from_head true
      <parse>
        @type json
        time_key time
        time_format %Y-%m-%dT%H:%M:%S.%NZ
      </parse>
    </source>
    
    <match cms.**>
      @type elasticsearch
      host elasticsearch
      port 9200
      logstash_format true
      logstash_prefix cms
      flush_interval 5s
    </match>
```

## Backup Monitoring

### Backup Status Check
```yaml
# prometheus-rules/backup.yaml
groups:
- name: backup
  rules:
  - alert: BackupFailed
    expr: backup_success_timestamp < (time() - 86400)
    for: 1h
    labels:
      severity: critical
    annotations:
      summary: Backup has not completed successfully in 24 hours
```

## Performance Monitoring

### Resource Usage Rules
```yaml
# prometheus-rules/resources.yaml
groups:
- name: resources
  rules:
  - alert: HighMemoryUsage
    expr: container_memory_usage_bytes{container="cms"} > 900Mi
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: High memory usage detected
  
  - alert: HighCPUUsage
    expr: container_cpu_usage_seconds_total{container="cms"} > 0.8
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: High CPU usage detected
``` 