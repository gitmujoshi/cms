# Digital Contract Management System Infrastructure Configuration
# This file defines the core AWS infrastructure components required to run the DCMS application.

# Configure Terraform and providers
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"  # Using AWS provider version 5.x
    }
  }
  # Store Terraform state in S3 for team collaboration and state locking
  backend "s3" {
    bucket = "contract-management-terraform-state"
    key    = "state/terraform.tfstate"
    region = "us-west-2"
  }
}

# Configure AWS Provider with specified region
provider "aws" {
  region = var.aws_region
}

# Network Configuration
# --------------------

# VPC for isolating DCMS resources
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true  # Enable DNS hostnames for RDS connectivity
  enable_dns_support   = true  # Enable DNS support for name resolution

  tags = {
    Name = "${var.project_name}-vpc"
  }
}

# Public subnets for the application load balancer and ECS tasks
resource "aws_subnet" "public" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index)  # Divide VPC CIDR into /24 subnets
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name = "${var.project_name}-public-${count.index + 1}"
  }
}

# Container Orchestration
# ----------------------

# ECS Cluster for running containerized DCMS application
resource "aws_ecs_cluster" "main" {
  name = "${var.project_name}-cluster"

  # Enable CloudWatch Container Insights for monitoring
  setting {
    name  = "containerInsights"
    value = "enabled"
  }
}

# ECS Task Definition specifying container configuration
resource "aws_ecs_task_definition" "app" {
  family                   = "${var.project_name}-app"
  requires_compatibilities = ["FARGATE"]  # Use serverless Fargate launch type
  network_mode            = "awsvpc"      # Required for Fargate
  cpu                     = var.task_cpu
  memory                  = var.task_memory

  # Container definition with environment variables and logging
  container_definitions = jsonencode([
    {
      name  = "${var.project_name}-app"
      image = "${var.ecr_repository_url}:latest"
      portMappings = [
        {
          containerPort = 8080
          hostPort      = 8080
          protocol      = "tcp"
        }
      ]
      # Environment variables for application configuration
      environment = [
        {
          name  = "DATABASE_URL"
          value = "postgresql://${aws_db_instance.main.username}:${aws_db_instance.main.password}@${aws_db_instance.main.endpoint}/${aws_db_instance.main.db_name}"
        },
        {
          name  = "AWS_REGION"
          value = var.aws_region
        }
      ]
      # CloudWatch logs configuration
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          awslogs-group         = "/ecs/${var.project_name}"
          awslogs-region       = var.aws_region
          awslogs-stream-prefix = "ecs"
        }
      }
    }
  ])
}

# Database Configuration
# ---------------------

# RDS PostgreSQL instance for persistent storage
resource "aws_db_instance" "main" {
  identifier           = "${var.project_name}-db"
  allocated_storage    = 20
  storage_type        = "gp2"              # General Purpose SSD
  engine              = "postgres"
  engine_version      = "15.3"             # PostgreSQL version
  instance_class      = var.db_instance_class
  db_name             = "contract_management"
  username            = var.db_username
  password            = var.db_password
  skip_final_snapshot = true               # Skip final snapshot when destroying

  vpc_security_group_ids = [aws_security_group.db.id]
  db_subnet_group_name   = aws_db_subnet_group.main.name

  backup_retention_period = 7              # Keep backups for 7 days
  multi_az               = var.environment == "production"  # Enable Multi-AZ for production

  tags = {
    Name = "${var.project_name}-db"
  }
}

# Load Balancer Configuration
# --------------------------

# Application Load Balancer for distributing traffic
resource "aws_lb" "main" {
  name               = "${var.project_name}-alb"
  internal           = false               # Internet-facing
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets           = aws_subnet.public[*].id

  tags = {
    Name = "${var.project_name}-alb"
  }
}

# ECS Service Configuration
# ------------------------

# ECS Service for running and maintaining desired task count
resource "aws_ecs_service" "main" {
  name            = "${var.project_name}-service"
  cluster         = aws_ecs_cluster.main.id
  task_definition = aws_ecs_task_definition.app.arn
  desired_count   = var.app_count
  launch_type     = "FARGATE"

  # Network configuration for tasks
  network_configuration {
    security_groups = [aws_security_group.app.id]
    subnets         = aws_subnet.public[*].id
  }

  # Load balancer target group attachment
  load_balancer {
    target_group_arn = aws_lb_target_group.app.arn
    container_name   = "${var.project_name}-app"
    container_port   = 8080
  }

  depends_on = [aws_lb_listener.front_end]
}

# Monitoring Configuration
# -----------------------

# CloudWatch Log Group for container logs
resource "aws_cloudwatch_log_group" "ecs" {
  name              = "/ecs/${var.project_name}"
  retention_in_days = 30                   # Keep logs for 30 days
}

# Security Configuration
# ---------------------

# Security group for application containers
resource "aws_security_group" "app" {
  name        = "${var.project_name}-app-sg"
  description = "Security group for app containers"
  vpc_id      = aws_vpc.main.id

  # Allow inbound traffic from ALB only
  ingress {
    from_port       = 8080
    to_port         = 8080
    protocol        = "tcp"
    security_groups = [aws_security_group.alb.id]
  }

  # Allow all outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# Security group for Application Load Balancer
resource "aws_security_group" "alb" {
  name        = "${var.project_name}-alb-sg"
  description = "Security group for ALB"
  vpc_id      = aws_vpc.main.id

  # Allow HTTP traffic
  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Allow HTTPS traffic
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  # Allow all outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
}

# Security group for RDS database
resource "aws_security_group" "db" {
  name        = "${var.project_name}-db-sg"
  description = "Security group for database"
  vpc_id      = aws_vpc.main.id

  # Allow PostgreSQL traffic from application only
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
  }
} 