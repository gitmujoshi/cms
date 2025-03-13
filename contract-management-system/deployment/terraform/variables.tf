# Variables Configuration for Digital Contract Management System
# This file defines all variables used in the Terraform configuration

# Project Configuration
# --------------------

variable "project_name" {
  description = "Name of the project, used as a prefix for all resources"
  type        = string
  default     = "contract-management"
}

variable "aws_region" {
  description = "AWS region where resources will be deployed"
  type        = string
  default     = "us-west-2"
}

variable "environment" {
  description = "Deployment environment (development, staging, production) - affects resource configuration"
  type        = string
  default     = "development"
}

# Network Configuration
# --------------------

variable "vpc_cidr" {
  description = "CIDR block for the VPC network space"
  type        = string
  default     = "10.0.0.0/16"  # Allows for up to 65,536 IP addresses
}

variable "availability_zones" {
  description = "List of AWS availability zones for high availability"
  type        = list(string)
  default     = ["us-west-2a", "us-west-2b", "us-west-2c"]  # Multi-AZ deployment
}

# Container Configuration
# ----------------------

variable "task_cpu" {
  description = "CPU units for the ECS task (1 vCPU = 1024 units)"
  type        = number
  default     = 256  # 0.25 vCPU
}

variable "task_memory" {
  description = "Memory (in MiB) for the ECS task"
  type        = number
  default     = 512  # 0.5 GB
}

variable "app_count" {
  description = "Number of application instances to run for high availability"
  type        = number
  default     = 2    # Minimum 2 instances for redundancy
}

# Database Configuration
# ---------------------

variable "db_instance_class" {
  description = "RDS instance type determining compute and memory capacity"
  type        = string
  default     = "db.t3.micro"  # Suitable for development; increase for production
}

variable "db_username" {
  description = "Master username for the RDS instance"
  type        = string
  sensitive   = true  # Marked as sensitive to prevent exposure in logs
}

variable "db_password" {
  description = "Master password for the RDS instance"
  type        = string
  sensitive   = true  # Marked as sensitive to prevent exposure in logs
}

# Container Registry Configuration
# ------------------------------

variable "ecr_repository_url" {
  description = "URL of the ECR repository where application container images are stored"
  type        = string
} 