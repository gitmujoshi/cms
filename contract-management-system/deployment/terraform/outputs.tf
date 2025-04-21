# Terraform Outputs Configuration for Digital Contract Management System
# This file defines all output values that are useful for reference after deployment
# Outputs can be used to extract information about the deployed infrastructure

# Load Balancer Outputs
# --------------------
# These outputs provide information about the Application Load Balancer

output "alb_dns_name" {
  description = "DNS name of the application load balancer for accessing the application"
  value       = aws_lb.main.dns_name
  # This DNS name can be used to access the application
  # Example: http://<alb_dns_name>
}

output "alb_zone_id" {
  description = "The canonical hosted zone ID of the load balancer"
  value       = aws_lb.main.zone_id
  # This can be used when creating Route 53 records
}

# ECS Outputs
# -----------
# These outputs provide information about the ECS cluster and services

output "ecs_cluster_name" {
  description = "Name of the ECS cluster for container deployment and management"
  value       = aws_ecs_cluster.main.name
  # This name can be used to reference the cluster in AWS CLI or SDK commands
}

output "ecs_service_name" {
  description = "Name of the ECS service running the application"
  value       = aws_ecs_service.main.name
  # This name can be used to manage the service (e.g., update task count)
}

# Database Outputs
# ---------------
# These outputs provide information about the RDS instance

output "rds_endpoint" {
  description = "Connection endpoint for the RDS PostgreSQL instance"
  value       = aws_db_instance.main.endpoint
  # This endpoint can be used to connect to the database
  # Format: <endpoint>:5432
}

output "rds_identifier" {
  description = "The RDS instance identifier"
  value       = aws_db_instance.main.identifier
  # This identifier can be used to reference the database in AWS CLI or SDK commands
}

# Network Outputs
# --------------
# These outputs provide information about the VPC and subnets

output "vpc_id" {
  description = "ID of the VPC where all resources are deployed"
  value       = aws_vpc.main.id
  # This ID can be used to reference the VPC in other Terraform configurations
}

output "public_subnet_ids" {
  description = "IDs of the public subnets used for the application and load balancer"
  value       = aws_subnet.public[*].id
  # These IDs can be used to deploy additional resources in the same subnets
}

# Security Group Outputs
# ---------------------
# These outputs provide information about the security groups

output "app_security_group_id" {
  description = "ID of the security group attached to the application containers"
  value       = aws_security_group.app.id
  # This ID can be used to reference the security group in other configurations
}

output "alb_security_group_id" {
  description = "ID of the security group attached to the application load balancer"
  value       = aws_security_group.alb.id
  # This ID can be used to reference the security group in other configurations
}

output "db_security_group_id" {
  description = "ID of the security group attached to the RDS instance"
  value       = aws_security_group.db.id
  # This ID can be used to reference the security group in other configurations
}

# Monitoring Outputs
# -----------------
# These outputs provide information about monitoring resources

output "cloudwatch_log_group" {
  description = "Name of the CloudWatch log group for container logs"
  value       = aws_cloudwatch_log_group.ecs.name
  # This name can be used to view logs in the AWS Console or through the CLI
}

output "cloudwatch_log_group_arn" {
  description = "ARN of the CloudWatch log group for container logs"
  value       = aws_cloudwatch_log_group.ecs.arn
  # This ARN can be used to grant permissions to other AWS services
}

# Additional Useful Outputs
# ------------------------
# These outputs provide additional useful information

output "ecr_repository_url" {
  description = "URL of the ECR repository for the application container"
  value       = var.ecr_repository_url
  # This URL can be used to push new container images
}

output "environment" {
  description = "The deployment environment (development, staging, production)"
  value       = var.environment
  # This can be used to verify the current environment
}

output "aws_region" {
  description = "The AWS region where resources are deployed"
  value       = var.aws_region
  # This can be used to verify the current region
} 