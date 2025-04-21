# Alerting Documentation

## Overview

This document details the alerting system configuration, rules, and management for the Contract Management System.

## Alerting Components

### Alertmanager Configuration
```yaml
global:
  resolve_timeout: 5m
  slack_api_url: 'https://hooks.slack.com/services/...'

route:
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  receiver: 'slack-notifications'
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty-notifications'
```

### Alert Channels

1. **Slack Integration**
   ```yaml
   receivers:
     - name: 'slack-notifications'
       slack_configs:
         - channel: '#alerts'
           send_resolved: true
           title: '{{ template "slack.default.title" . }}'
           text: '{{ template "slack.default.text" . }}'
   ```

2. **PagerDuty Integration**
   ```yaml
   receivers:
     - name: 'pagerduty-notifications'
       pagerduty_configs:
         - routing_key: 'your-routing-key'
           send_resolved: true
   ```

3. **Email Notifications**
   ```yaml
   receivers:
     - name: 'email-notifications'
       email_configs:
         - to: 'alerts@example.com'
           from: 'alertmanager@example.com'
           smarthost: 'smtp.example.com:587'
           auth_username: 'alertmanager'
   ```

## Alert Rules

### System Alerts

1. **High Resource Usage**
   ```yaml
   - alert: HighCPUUsage
     expr: node_cpu_seconds_total{mode="user"} > 80
     for: 5m
     labels:
       severity: warning
     annotations:
       summary: High CPU usage detected
       description: CPU usage is above 80% for 5 minutes

   - alert: HighMemoryUsage
     expr: (node_memory_MemTotal_bytes - node_memory_MemFree_bytes) / node_memory_MemTotal_bytes * 100 > 85
     for: 5m
     labels:
       severity: warning
     annotations:
       summary: High memory usage detected
   ```

2. **Disk Space**
   ```yaml
   - alert: LowDiskSpace
     expr: node_filesystem_free_bytes / node_filesystem_size_bytes * 100 < 10
     for: 5m
     labels:
       severity: warning
     annotations:
       summary: Low disk space
   ```

### Application Alerts

1. **API Performance**
   ```yaml
   - alert: HighAPIErrorRate
     expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
     for: 5m
     labels:
       severity: critical
     annotations:
       summary: High API error rate

   - alert: SlowAPIResponse
     expr: http_request_duration_seconds{quantile="0.95"} > 2
     for: 5m
     labels:
       severity: warning
     annotations:
       summary: Slow API responses
   ```

2. **Contract Operations**
   ```yaml
   - alert: HighContractCreationFailure
     expr: rate(contract_creation_failed_total[5m]) > 0.05
     for: 5m
     labels:
       severity: critical
     annotations:
       summary: High contract creation failure rate

   - alert: ExpiringContracts
     expr: contracts_by_status{status="active"} * 0.1
     for: 1h
     labels:
       severity: warning
     annotations:
       summary: Contracts approaching expiration
   ```

### Business Alerts

1. **User Activity**
   ```yaml
   - alert: UnusualUserActivity
     expr: abs(rate(user_actions_total[1h]) - rate(user_actions_total[1h] offset 1d)) > 0.5
     for: 30m
     labels:
       severity: warning
     annotations:
       summary: Unusual user activity pattern detected

   - alert: HighFailedLogins
     expr: rate(failed_logins_total[5m]) > 5
     for: 5m
     labels:
       severity: critical
     annotations:
       summary: High number of failed login attempts
   ```

## Alert Management

### Alert Lifecycle

1. **Alert Creation**
   - Define alert conditions
   - Set severity levels
   - Configure notification channels

2. **Alert Processing**
   - Group similar alerts
   - Apply inhibition rules
   - Route to appropriate channels

3. **Alert Resolution**
   - Mark alerts as resolved
   - Send resolution notifications
   - Update alert history

### Alert Suppression

```yaml
inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname', 'instance']
```

## Best Practices

1. **Alert Design**
   - Use meaningful alert names
   - Include clear descriptions
   - Set appropriate thresholds
   - Define proper severity levels

2. **Notification Management**
   - Group related alerts
   - Use appropriate channels
   - Include relevant context
   - Set proper timing

3. **Maintenance**
   - Regularly review alerts
   - Update thresholds
   - Clean up unused alerts
   - Document changes

## Troubleshooting

### Common Issues

1. **Missing Alerts**
   - Check Prometheus rules
   - Verify Alertmanager config
   - Check notification channels
   - Review logs

2. **Alert Fatigue**
   - Review alert thresholds
   - Consolidate similar alerts
   - Update notification policies
   - Implement proper grouping

3. **False Positives**
   - Adjust thresholds
   - Add additional conditions
   - Review alert logic
   - Update documentation

## Additional Resources

- [Alertmanager Documentation](https://prometheus.io/docs/alerting/latest/alertmanager/)
- [Prometheus Alerting Rules](https://prometheus.io/docs/prometheus/latest/configuration/alerting_rules/)
- [Alerting Best Practices](https://prometheus.io/docs/practices/alerting/) 