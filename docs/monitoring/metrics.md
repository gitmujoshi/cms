# Metrics Documentation

## Overview

This document details the metrics collected by the Contract Management System, including system, application, and business metrics.

## Metric Categories

### 1. System Metrics

#### CPU Metrics
```prometheus
# CPU Usage
node_cpu_seconds_total{mode="user"}
node_cpu_seconds_total{mode="system"}
node_cpu_seconds_total{mode="idle"}

# CPU Load
node_load1
node_load5
node_load15
```

#### Memory Metrics
```prometheus
# Memory Usage
node_memory_MemTotal_bytes
node_memory_MemFree_bytes
node_memory_Buffers_bytes
node_memory_Cached_bytes
```

#### Disk Metrics
```prometheus
# Disk Usage
node_filesystem_size_bytes
node_filesystem_free_bytes
node_filesystem_avail_bytes

# Disk I/O
node_disk_read_bytes_total
node_disk_written_bytes_total
node_disk_io_time_seconds_total
```

#### Network Metrics
```prometheus
# Network Traffic
node_network_receive_bytes_total
node_network_transmit_bytes_total
node_network_receive_packets_total
node_network_transmit_packets_total
```

### 2. Application Metrics

#### API Metrics
```prometheus
# Request Count
http_requests_total{method="POST",endpoint="/api/contracts"}
http_requests_total{method="GET",endpoint="/api/contracts"}

# Response Time
http_request_duration_seconds{method="POST",endpoint="/api/contracts"}
http_request_duration_seconds{method="GET",endpoint="/api/contracts"}

# Error Rate
http_requests_total{status="500"}
http_requests_total{status="400"}
```

#### Contract Metrics
```prometheus
# Contract Operations
contract_created_total
contract_signed_total
contract_updated_total
contract_deleted_total

# Contract Status
contracts_by_status{status="draft"}
contracts_by_status{status="active"}
contracts_by_status{status="expired"}
```

#### Document Metrics
```prometheus
# Document Operations
document_uploaded_total
document_downloaded_total
document_deleted_total

# Storage Usage
document_storage_bytes_total
document_storage_bytes_by_type{type="pdf"}
document_storage_bytes_by_type{type="doc"}
```

### 3. Business Metrics

#### User Activity
```prometheus
# User Sessions
user_sessions_total
active_users_total
failed_logins_total

# User Actions
user_actions_total{action="create_contract"}
user_actions_total{action="sign_contract"}
user_actions_total{action="upload_document"}
```

#### Performance Metrics
```prometheus
# System Performance
system_uptime_seconds
api_response_time_seconds
database_query_time_seconds

# Resource Utilization
resource_utilization_percent{cpu="true"}
resource_utilization_percent{memory="true"}
resource_utilization_percent{disk="true"}
```

## Metric Collection

### Prometheus Configuration
```yaml
scrape_configs:
  - job_name: 'contract-management'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: '/metrics'
```

### Metric Labels
- `environment`: prod, staging, dev
- `service`: api, database, cache
- `instance`: hostname or IP
- `version`: application version

## Metric Visualization

### Grafana Dashboards

1. **System Overview**
   - CPU, Memory, Disk usage
   - Network traffic
   - System load

2. **Application Performance**
   - API response times
   - Error rates
   - Request volumes

3. **Business Metrics**
   - Contract statistics
   - User activity
   - Document storage

## Alerting Rules

### System Alerts
```yaml
groups:
  - name: system
    rules:
      - alert: HighCPUUsage
        expr: node_cpu_seconds_total{mode="user"} > 80
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: High CPU usage detected
```

### Application Alerts
```yaml
groups:
  - name: application
    rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate detected
```

## Best Practices

1. **Metric Naming**
   - Use consistent naming conventions
   - Include units in metric names
   - Use descriptive labels

2. **Cardinality Management**
   - Limit label combinations
   - Use appropriate aggregation
   - Monitor metric growth

3. **Performance Impact**
   - Optimize scrape intervals
   - Use efficient queries
   - Monitor resource usage

## Troubleshooting

### Common Issues

1. **Missing Metrics**
   - Check scrape configuration
   - Verify metric endpoints
   - Check network connectivity

2. **High Cardinality**
   - Review label usage
   - Implement aggregation
   - Clean up unused metrics

3. **Performance Issues**
   - Optimize queries
   - Adjust scrape intervals
   - Monitor resource usage

## Additional Resources

- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [Grafana Dashboard Examples](https://grafana.com/grafana/dashboards/)
- [Metric Collection Guidelines](https://prometheus.io/docs/instrumenting/writing_exporters/) 