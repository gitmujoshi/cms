# Digital Contract Management System (DCMS) - Infrastructure Guide

## Overview

The Digital Contract Management System is deployed on AWS using a modern, containerized architecture. This guide details the infrastructure setup and deployment process using Terraform.

## Architecture

The DCMS infrastructure consists of the following components:

- **Compute Layer**:
  - ECS Fargate for containerized application deployment
  - Multiple instances for high availability
  - Auto-scaling capabilities

- **Database Layer**:
  - Amazon RDS PostgreSQL
  - Automated backups
  - Optional Multi-AZ deployment for production

- **Network Layer**:
  - Custom VPC with public subnets
  - Application Load Balancer for traffic distribution
  - Security groups for network isolation

- **Monitoring**:
  - CloudWatch Container Insights
  - Centralized logging
  - Application metrics collection

## Prerequisites

1. **AWS Account and Credentials**:
   - AWS account with administrative access
   - AWS CLI installed and configured
   - Access and secret keys with appropriate permissions

2. **Required Tools**:
   - Terraform >= 1.0.0
   - AWS CLI >= 2.0.0
   - Docker (for building application images)

3. **Infrastructure Dependencies**:
   - S3 bucket for Terraform state (specified in `main.tf`)
   - ECR repository for container images

## Initial Setup

1. **Configure AWS Credentials**:
   ```bash
   aws configure
   ```

2. **Create S3 Bucket for Terraform State**:
   ```bash
   aws s3 mb s3://contract-management-terraform-state
   aws s3api put-bucket-versioning \
     --bucket contract-management-terraform-state \
     --versioning-configuration Status=Enabled
   ```

3. **Create ECR Repository**:
   ```bash
   aws ecr create-repository \
     --repository-name contract-management \
     --image-scanning-configuration scanOnPush=true
   ```

## Configuration

1. **Environment Variables**:
   Create a `terraform.tfvars` file:
   ```hcl
   project_name        = "contract-management"
   environment         = "development"  # or "production"
   aws_region         = "us-west-2"
   db_username        = "your_db_username"
   db_password        = "your_secure_password"
   ecr_repository_url = "your.ecr.repository.url"
   ```

2. **Production Settings**:
   For production deployments, consider adjusting:
   ```hcl
   environment         = "production"
   db_instance_class   = "db.t3.small"  # or larger
   task_cpu           = 512             # 0.5 vCPU
   task_memory        = 1024            # 1 GB
   app_count          = 3               # More instances for HA
   ```

## Deployment Steps

1. **Initialize Terraform**:
   ```bash
   cd deployment/terraform
   terraform init
   ```

2. **Plan the Deployment**:
   ```bash
   terraform plan -out=tfplan
   ```
   Review the plan carefully to ensure all resources are configured as expected.

3. **Apply the Configuration**:
   ```bash
   terraform apply tfplan
   ```

4. **Verify Deployment**:
   ```bash
   terraform output
   ```
   Note down the important outputs like ALB DNS name and database endpoint.

## Post-Deployment Configuration

1. **Database Migration**:
   ```bash
   # Set DATABASE_URL environment variable
   export DATABASE_URL="postgresql://$(terraform output -raw rds_endpoint)"
   
   # Run migrations
   cargo run --bin migrations
   ```

2. **Application Deployment**:
   ```bash
   # Build and push Docker image
   docker build -t contract-management .
   docker tag contract-management:latest $ECR_REPO_URL:latest
   docker push $ECR_REPO_URL:latest
   ```

3. **Access the Application**:
   - Use the ALB DNS name from `terraform output alb_dns_name`
   - Configure DNS if needed
   - Verify application health checks

## Monitoring and Maintenance

1. **Logs**:
   - Access container logs in CloudWatch:
     ```bash
     aws logs get-log-events \
       --log-group-name "/ecs/contract-management" \
       --log-stream-name "your-stream-name"
     ```

2. **Metrics**:
   - Monitor ECS metrics in CloudWatch Container Insights
   - Set up CloudWatch Alarms for key metrics
   - Configure notification endpoints

3. **Backups**:
   - RDS automated backups are enabled (7-day retention)
   - Manual snapshots recommended before major changes

## Security Considerations

1. **Network Security**:
   - All traffic is encrypted in transit
   - Database accessible only from application
   - ALB configured for HTTPS (requires certificate)

2. **Access Control**:
   - Use IAM roles for service access
   - Rotate database credentials regularly
   - Monitor security group changes

3. **Compliance**:
   - Enable AWS Config for resource tracking
   - Set up CloudTrail for API auditing
   - Regular security assessments

## Troubleshooting

1. **Common Issues**:
   - Database connection failures
     - Check security group rules
     - Verify endpoint and credentials
   - Container deployment issues
     - Review ECS task logs
     - Check ECR image availability

2. **Health Checks**:
   - ALB target group health
   - ECS service status
   - RDS instance status

## Infrastructure Updates

1. **Making Changes**:
   ```bash
   # Update configuration
   terraform plan -out=tfplan
   terraform apply tfplan
   ```

2. **Version Control**:
   - Commit all Terraform changes
   - Document major updates
   - Use meaningful commit messages

## Cleanup

To destroy the infrastructure (use with caution):
```bash
terraform destroy
```

## Support and Documentation

- [AWS ECS Documentation](https://docs.aws.amazon.com/ecs/)
- [AWS RDS Documentation](https://docs.aws.amazon.com/rds/)
- [Terraform AWS Provider](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Submit a pull request with detailed description

## License

This infrastructure code is licensed under the MIT License. See LICENSE file for details. 