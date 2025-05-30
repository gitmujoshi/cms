# Variables Configuration for Digital Contract Management System
# This file defines all variables used in the Terraform configuration
# Variables allow for customization of the infrastructure without modifying the main configuration

# Project Configuration
# --------------------
# These variables define the basic project settings and environment

variable "project_name" {
  description = "Name of the project, used as a prefix for all resources"
  type        = string
  default     = "contract-management"
  # This value is used to create unique resource names and tags
}

variable "aws_region" {
  description = "AWS region where resources will be deployed"
  type        = string
  default     = "us-east-2"
  # Choose a region close to your users for better performance
  # Available regions: https://docs.aws.amazon.com/general/latest/gr/rande.html
}

variable "environment" {
  description = "Deployment environment (development, staging, production) - affects resource configuration"
  type        = string
  default     = "development"
  # This variable controls resource sizing and features like Multi-AZ deployment
  # Valid values: development, staging, production
}

# Network Configuration
# --------------------
# These variables control the network topology and IP addressing

variable "vpc_cidr" {
  description = "CIDR block for the VPC network space"
  type        = string
  default     = "10.0.0.0/16"  # Allows for up to 65,536 IP addresses
  # This CIDR block should be large enough to accommodate all subnets
  # and should not overlap with other VPCs you might want to peer with
}

variable "availability_zones" {
  description = "List of AWS availability zones for high availability"
  type        = list(string)
  default     = ["us-west-2a", "us-west-2b", "us-west-2c"]  # Multi-AZ deployment
  # Using multiple AZs provides high availability and fault tolerance
  # Choose AZs that are in the same region as specified in aws_region
}

# Container Configuration
# ----------------------
# These variables control the ECS task and container configuration

variable "task_cpu" {
  description = "CPU units for the ECS task (1 vCPU = 1024 units)"
  type        = number
  default     = 256  # 0.25 vCPU
  # Adjust based on your application's CPU requirements
  # Valid values: 256 (0.25 vCPU), 512 (0.5 vCPU), 1024 (1 vCPU), etc.
}

variable "task_memory" {
  description = "Memory (in MiB) for the ECS task"
  type        = number
  default     = 512  # 0.5 GB
  # Adjust based on your application's memory requirements
  # The amount of memory must be compatible with the CPU value
}

variable "app_count" {
  description = "Number of application instances to run for high availability"
  type        = number
  default     = 2    # Minimum 2 instances for redundancy
  # This value should be at least 2 in production for high availability
  # Can be adjusted based on expected load and desired redundancy
}

# Database Configuration
# ---------------------
# These variables control the RDS instance configuration

variable "db_instance_class" {
  description = "RDS instance type determining compute and memory capacity"
  type        = string
  default     = "db.t3.micro"  # Suitable for development; increase for production
  # Choose an instance class based on your database workload
  # Development: db.t3.micro
  # Production: db.r5.large or larger
  # Available instance types: https://aws.amazon.com/rds/instance-types/
}

variable "db_username" {
  description = "Master username for the RDS instance"
  type        = string
  sensitive   = true  # Marked as sensitive to prevent exposure in logs
  # This username will have full access to the database
  # Should be provided through environment variables or secrets management
}

variable "db_password" {
  description = "Master password for the RDS instance"
  type        = string
  sensitive   = true  # Marked as sensitive to prevent exposure in logs
  # This password should be strong and unique
  # Should be provided through environment variables or secrets management
  # Consider using AWS Secrets Manager for production environments
}

# Container Registry Configuration
# ------------------------------
# These variables control the ECR repository configuration

variable "ecr_repository_url" {
  description = "URL of the ECR repository where application container images are stored"
  type        = string
  # This should be the full URL of your ECR repository
  # Format: <account-id>.dkr.ecr.<region>.amazonaws.com/<repository-name>
}

# Monitoring Configuration
# ----------------------
# These variables control the monitoring and logging setup

variable "log_retention_days" {
  description = "Number of days to retain CloudWatch logs"
  type        = number
  default     = 30
  # Adjust based on your compliance requirements
  # Common values: 30 (development), 90 (staging), 365 (production)
}

variable "enable_enhanced_monitoring" {
  description = "Whether to enable enhanced monitoring for RDS"
  type        = bool
  default     = true
  # Enhanced monitoring provides more detailed metrics
  # Recommended for production environments
}

# Security Configuration
# ---------------------
# These variables control security-related settings

variable "enable_encryption" {
  description = "Whether to enable encryption at rest for RDS and EBS volumes"
  type        = bool
  default     = true
  # Encryption is recommended for all environments
  # Required for compliance with many security standards
}

variable "allowed_cidr_blocks" {
  description = "List of CIDR blocks allowed to access the application"
  type        = list(string)
  default     = ["0.0.0.0/0"]  # Allow access from anywhere
  # In production, restrict this to your organization's IP ranges
  # Example: ["10.0.0.0/8", "192.168.0.0/16"]
} 