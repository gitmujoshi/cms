# Terraform Outputs Configuration for Digital Contract Management System
# This file defines all output values that are useful for reference after deployment

# Load Balancer Outputs
# --------------------

output "alb_dns_name" {
  description = "DNS name of the application load balancer for accessing the application"
  value       = aws_lb.main.dns_name
}

# ECS Outputs
# -----------

output "ecs_cluster_name" {
  description = "Name of the ECS cluster for container deployment and management"
  value       = aws_ecs_cluster.main.name
}

# Database Outputs
# ---------------

output "rds_endpoint" {
  description = "Connection endpoint for the RDS PostgreSQL instance"
  value       = aws_db_instance.main.endpoint
}

# Network Outputs
# --------------

output "vpc_id" {
  description = "ID of the VPC where all resources are deployed"
  value       = aws_vpc.main.id
}

output "public_subnet_ids" {
  description = "IDs of the public subnets used for the application and load balancer"
  value       = aws_subnet.public[*].id
}

# Security Group Outputs
# ---------------------

output "app_security_group_id" {
  description = "ID of the security group attached to the application containers"
  value       = aws_security_group.app.id
}

output "alb_security_group_id" {
  description = "ID of the security group attached to the application load balancer"
  value       = aws_security_group.alb.id
}

output "db_security_group_id" {
  description = "ID of the security group attached to the RDS instance"
  value       = aws_security_group.db.id
}

# Monitoring Outputs
# -----------------

output "cloudwatch_log_group" {
  description = "Name of the CloudWatch log group for container logs"
  value       = aws_cloudwatch_log_group.ecs.name
} 