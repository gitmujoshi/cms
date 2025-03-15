# Remote Deployment Guide (MacOS)

## Prerequisites

1. **Install Required Tools**:
   ```bash
   # Install Homebrew if not already installed
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

   # Install development tools
   brew install rust
   brew install aws-cli
   brew install terraform
   brew install docker
   brew install postgresql@14  # For local development

   # Install Cargo tools
   cargo install cargo-watch
   cargo install sea-orm-cli
   ```

2. **Configure AWS Credentials**:
   ```bash
   aws configure
   # Enter your:
   # - AWS Access Key ID
   # - AWS Secret Access Key
   # - Default region (e.g., us-west-2)
   # - Output format (json)
   ```

3. **Clone Repository**:
   ```bash
   git clone https://github.com/your-org/contract-management-system.git
   cd contract-management-system
   ```

## Local Development Setup

1. **Environment Setup**:
   ```bash
   # Create .env file
   cat > .env << EOL
   DATABASE_URL=postgresql://localhost:5432/contract_management
   AWS_REGION=us-west-2
   RUST_LOG=debug
   EOL

   # Start PostgreSQL service
   brew services start postgresql@14
   
   # Create local database
   createdb contract_management
   ```

2. **Build Project**:
   ```bash
   # Build all components
   cargo build --release

   # Run database migrations
   cargo run --bin migrations
   ```

3. **Local Testing**:
   ```bash
   # Run test suite
   cargo test

   # Start local development server
   cargo run --bin server
   ```

## Production Deployment

### 1. Infrastructure Deployment

1. **Initialize Terraform**:
   ```bash
   cd deployment/terraform
   
   # Initialize Terraform
   terraform init
   
   # Create production workspace
   terraform workspace new production
   
   # Review deployment plan
   terraform plan -out=tfplan -var-file=environments/production.tfvars
   
   # Apply infrastructure changes
   terraform apply tfplan
   ```

2. **Configure Production Environment**:
   ```bash
   # Export infrastructure outputs
   export DATABASE_URL="$(terraform output -raw rds_endpoint)"
   export ECR_REPOSITORY="$(terraform output -raw ecr_repository_url)"
   export ECS_CLUSTER="$(terraform output -raw ecs_cluster_name)"
   ```

### 2. Application Deployment

1. **Build Docker Image**:
   ```bash
   # Authenticate with ECR
   aws ecr get-login-password --region us-west-2 | docker login --username AWS --password-stdin $ECR_REPOSITORY

   # Build and tag image
   docker build -t contract-management .
   docker tag contract-management:latest $ECR_REPOSITORY:latest

   # Push to ECR
   docker push $ECR_REPOSITORY:latest
   ```

2. **Deploy Application**:
   ```bash
   # Update ECS service
   aws ecs update-service \
     --cluster $ECS_CLUSTER \
     --service contract-management \
     --force-new-deployment
   ```

3. **Run Database Migrations**:
   ```bash
   # Run migrations in production
   cargo run --bin migrations -- --database-url $DATABASE_URL
   ```

### 3. Verify Deployment

1. **Check Service Status**:
   ```bash
   # Monitor deployment
   aws ecs describe-services \
     --cluster $ECS_CLUSTER \
     --services contract-management

   # View logs
   aws logs tail /ecs/contract-management --follow
   ```

2. **Monitor Health**:
   ```bash
   # Get load balancer URL
   export ALB_URL="$(terraform output -raw alb_dns_name)"
   
   # Check health endpoint
   curl -v "https://$ALB_URL/health"
   ```

## Monitoring and Maintenance

### 1. View Logs and Metrics

```bash
# View application logs
aws logs tail /ecs/contract-management --follow

# View metrics in CloudWatch
aws cloudwatch get-metric-statistics \
  --namespace AWS/ECS \
  --metric-name CPUUtilization \
  --dimensions Name=ClusterName,Value=$ECS_CLUSTER \
  --start-time $(date -v-1H +%s) \
  --end-time $(date +%s) \
  --period 300 \
  --statistics Average
```

### 2. Common Operations

1. **Scale Services**:
   ```bash
   # Scale task count
   aws ecs update-service \
     --cluster $ECS_CLUSTER \
     --service contract-management \
     --desired-count 3
   ```

2. **Update Application**:
   ```bash
   # Build and push new image
   docker build -t contract-management .
   docker tag contract-management:latest $ECR_REPOSITORY:latest
   docker push $ECR_REPOSITORY:latest
   
   # Deploy update
   aws ecs update-service \
     --cluster $ECS_CLUSTER \
     --service contract-management \
     --force-new-deployment
   ```

3. **Backup Database**:
   ```bash
   # Create snapshot
   aws rds create-db-snapshot \
     --db-instance-identifier contract-management \
     --db-snapshot-identifier backup-$(date +%Y%m%d)
   ```

## Troubleshooting

### Common Issues

1. **Docker Build Failures**:
   - Check Docker daemon is running: `docker info`
   - Clear Docker cache: `docker system prune -a`
   - Verify Dockerfile syntax

2. **Deployment Failures**:
   - Check ECS service events:
     ```bash
     aws ecs describe-services \
       --cluster $ECS_CLUSTER \
       --services contract-management
     ```
   - Verify task definition:
     ```bash
     aws ecs describe-task-definition \
       --task-definition contract-management
     ```
   - Check CloudWatch logs for errors

3. **Database Connection Issues**:
   - Verify security group rules
   - Check RDS instance status
   - Test connection locally:
     ```bash
     psql "$DATABASE_URL"
     ```

### Security Best Practices

1. **Credential Management**:
   - Use AWS Secrets Manager for sensitive data
   - Rotate credentials regularly
   - Never commit credentials to git

2. **Network Security**:
   - Use VPC endpoints where possible
   - Restrict security group access
   - Enable AWS WAF on ALB

3. **Monitoring**:
   - Set up CloudWatch alarms
   - Enable AWS Config
   - Use AWS CloudTrail

## Rollback Procedures

1. **Application Rollback**:
   ```bash
   # Deploy previous version
   aws ecs update-service \
     --cluster $ECS_CLUSTER \
     --service contract-management \
     --task-definition contract-management:PREVIOUS_VERSION
   ```

2. **Database Rollback**:
   ```bash
   # Revert last migration
   cargo run --bin migrations -- \
     --database-url $DATABASE_URL \
     --rollback
   ```

3. **Infrastructure Rollback**:
   ```bash
   # Revert to previous state
   terraform plan -destroy -out=destroy.tfplan
   terraform apply destroy.tfplan
   ``` 