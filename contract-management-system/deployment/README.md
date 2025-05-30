# Contract Management System Deployment Guide

## Overview

This repository contains Terraform configurations for deploying the Contract Management System on AWS. The infrastructure is designed to be highly available, secure, and scalable, using modern AWS services and best practices.

## Architecture

The system is deployed using the following AWS services:

- **ECS (Elastic Container Service)** with Fargate for container orchestration
- **RDS PostgreSQL** for persistent data storage
- **Application Load Balancer** for traffic distribution
- **VPC** with public and private subnets for network isolation
- **CloudWatch** for monitoring and logging
- **S3** for ALB access logs and Terraform state storage

## Prerequisites

1. **AWS Account**
   - An AWS account with appropriate permissions
   - AWS CLI configured with credentials
   - Required IAM permissions for deployment

2. **Tools**
   - Terraform (v1.0.0 or later)
   - AWS CLI (v2.0 or later)
   - Git

3. **Environment Variables**
   ```bash
   export AWS_ACCESS_KEY_ID="your_access_key"
   export AWS_SECRET_ACCESS_KEY="your_secret_key"
   export AWS_REGION="us-west-2"
   ```aws 

## Directory Structure

```
deployment/
├── terraform/
│   ├── main.tf          # Main infrastructure configuration
│   ├── variables.tf     # Variable definitions
│   ├── outputs.tf       # Output values
│   └── README.md        # Terraform-specific documentation
└── README.md            # This file
```

## Deployment Steps

1. **Clone the Repository**
   ```bash
   git clone <repository-url>
   cd contract-management-system/deployment
   ```

2. **Initialize Terraform**
   ```bash
   cd terraform
   terraform init
   ```

3. **Review and Modify Variables**
   - Edit `terraform.tfvars` or use environment variables
   - Key variables to configure:
     ```hcl
     project_name = "contract-management"
     environment  = "production"
     aws_region   = "us-west-2"
     ```

4. **Plan the Deployment**
   ```bash
   terraform plan -out=tfplan
   ```

5. **Apply the Configuration**
   ```bash
   terraform apply tfplan
   ```

## Environment-Specific Configurations

### Development Environment
```hcl
environment = "development"
app_count   = 1
db_instance_class = "db.t3.micro"
task_cpu    = 256
task_memory = 512
```

### Production Environment
```hcl
environment = "production"
app_count   = 3
db_instance_class = "db.r5.large"
task_cpu    = 1024
task_memory = 2048
multi_az    = true
```

## Security Considerations

1. **Network Security**
   - VPC with isolated subnets
   - Security groups with least privilege access
   - Network ACLs for additional protection

2. **Data Security**
   - RDS encryption at rest
   - SSL/TLS for data in transit
   - Regular security patches

3. **Access Control**
   - IAM roles with minimal permissions
   - Secrets management using AWS Secrets Manager
   - Regular access reviews

## Monitoring Setup

1. **CloudWatch Alarms**
   ```hcl
   resource "aws_cloudwatch_metric_alarm" "cpu_utilization" {
     alarm_name          = "${var.project_name}-cpu-utilization"
     comparison_operator = "GreaterThanThreshold"
     evaluation_periods  = "2"
     metric_name         = "CPUUtilization"
     namespace           = "AWS/ECS"
     period             = "300"
     statistic          = "Average"
     threshold          = "80"
     alarm_description  = "CPU utilization exceeds 80%"
   }
   ```

2. **Logging Configuration**
   - Container logs to CloudWatch
   - ALB access logs to S3
   - RDS audit logs

## Scaling Configuration

1. **ECS Auto Scaling**
   ```hcl
   resource "aws_appautoscaling_target" "ecs_target" {
     max_capacity       = 10
     min_capacity       = 2
     resource_id        = "service/${aws_ecs_cluster.main.name}/${aws_ecs_service.main.name}"
     scalable_dimension = "ecs:service:DesiredCount"
     service_namespace  = "ecs"
   }
   ```

2. **RDS Scaling**
   - Read replicas for high read throughput
   - Automatic storage scaling
   - Performance Insights enabled

## Backup and Recovery

1. **RDS Backups**
   - Automated daily backups
   - Point-in-time recovery
   - Cross-region replication

2. **Disaster Recovery**
   - Multi-AZ deployment
   - Automated failover
   - Regular DR testing

## Cost Optimization

1. **Resource Sizing**
   - Right-sized instances
   - Reserved instances for production
   - Spot instances for non-critical workloads

2. **Monitoring and Alerts**
   - Cost allocation tags
   - Budget alerts
   - Resource cleanup policies

## Troubleshooting

1. **Common Issues**
   - ECS task failures
   - RDS connection issues
   - ALB health check failures

2. **Debugging Tools**
   ```bash
   # Check ECS task status
   aws ecs describe-tasks --cluster ${var.project_name}-cluster --tasks <task-id>

   # View RDS logs
   aws rds describe-db-log-files --db-instance-identifier ${var.project_name}-db

   # Check ALB health
   aws elbv2 describe-target-health --target-group-arn <target-group-arn>
   ```

## Maintenance

1. **Regular Updates**
   - Terraform version updates
   - AWS provider updates
   - Security patches

2. **Backup Verification**
   - Regular backup testing
   - Recovery time objectives
   - Data integrity checks

## Support

For issues and questions:
1. Check the troubleshooting section
2. Review AWS documentation
3. Contact the development team

## License

This project is licensed under the MIT License - see the LICENSE file for details. 