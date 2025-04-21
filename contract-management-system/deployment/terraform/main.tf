# Digital Contract Management System Infrastructure Configuration
# This file defines the core AWS infrastructure components required to run the DCMS application.
# The infrastructure is designed to be highly available, secure, and scalable.

# Configure Terraform and providers
terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"  # Using AWS provider version 5.x for latest features and bug fixes
    }
  }
  # Store Terraform state in S3 for team collaboration and state locking
  # This enables multiple team members to work on the infrastructure safely
  backend "s3" {
    bucket = "contract-management-terraform-state"
    key    = "state/terraform.tfstate"
    region = "us-west-2"
    # Note: Enable versioning and encryption on the S3 bucket for security
    # Example bucket policy:
    # {
    #   "Version": "2012-10-17",
    #   "Statement": [
    #     {
    #       "Effect": "Deny",
    #       "Principal": "*",
    #       "Action": "s3:*",
    #       "Resource": "arn:aws:s3:::contract-management-terraform-state/*",
    #       "Condition": {
    #         "Bool": {
    #           "aws:SecureTransport": "false"
    #         }
    #       }
    #     }
    #   ]
    # }
  }
}

# Configure AWS Provider with specified region
# The provider block sets up authentication and default configuration for AWS resources
provider "aws" {
  region = var.aws_region
  # Note: AWS credentials should be configured using environment variables or AWS CLI
  # Example AWS CLI configuration:
  # aws configure
  # AWS Access Key ID: your_access_key
  # AWS Secret Access Key: your_secret_key
  # Default region name: us-west-2
  # Default output format: json
}

# Network Configuration
# --------------------
# The network configuration creates a VPC with public subnets for the application
# and private subnets for the database. This provides network isolation and security.

# VPC for isolating DCMS resources
# The VPC provides network isolation and security boundaries for all resources
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true  # Required for RDS endpoint resolution
  enable_dns_support   = true  # Required for internal DNS resolution

  tags = {
    Name = "${var.project_name}-vpc"
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# Internet Gateway for public subnet internet access
resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "${var.project_name}-igw"
    Environment = var.environment
  }
}

# Route table for public subnets
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name = "${var.project_name}-public-rt"
    Environment = var.environment
  }
}

# Public subnets for the application load balancer and ECS tasks
# These subnets have direct internet access through an Internet Gateway
resource "aws_subnet" "public" {
  count             = length(var.availability_zones)
  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index)  # Creates /24 subnets
  availability_zone = var.availability_zones[count.index]
  # Enable public IP assignment for instances launched in these subnets
  map_public_ip_on_launch = true

  tags = {
    Name = "${var.project_name}-public-${count.index + 1}"
    Environment = var.environment
    Tier = "public"
  }
}

# Associate public subnets with public route table
resource "aws_route_table_association" "public" {
  count          = length(var.availability_zones)
  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

# Container Orchestration
# ----------------------
# ECS (Elastic Container Service) configuration for running the application
# in containers with Fargate for serverless container execution.

# ECS Cluster for running containerized DCMS application
# The cluster provides the infrastructure for running containerized applications
resource "aws_ecs_cluster" "main" {
  name = "${var.project_name}-cluster"

  # Enable CloudWatch Container Insights for enhanced monitoring
  setting {
    name  = "containerInsights"
    value = "enabled"
  }

  # Example capacity provider configuration
  capacity_providers = ["FARGATE", "FARGATE_SPOT"]
  default_capacity_provider_strategy {
    capacity_provider = "FARGATE"
    weight            = 1
    base              = 1
  }

  tags = {
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# ECS Task Definition specifying container configuration
# Defines how the application container should run, including resources and networking
resource "aws_ecs_task_definition" "app" {
  family                   = "${var.project_name}-app"
  requires_compatibilities = ["FARGATE"]  # Use serverless Fargate launch type
  network_mode            = "awsvpc"      # Required for Fargate, provides enhanced networking
  cpu                     = var.task_cpu
  memory                  = var.task_memory
  execution_role_arn      = aws_iam_role.ecs_execution_role.arn
  task_role_arn          = aws_iam_role.ecs_task_role.arn

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
        },
        {
          name  = "ENVIRONMENT"
          value = var.environment
        },
        {
          name  = "LOG_LEVEL"
          value = var.environment == "production" ? "INFO" : "DEBUG"
        }
      ]
      # CloudWatch logs configuration for centralized logging
      logConfiguration = {
        logDriver = "awslogs"
        options = {
          awslogs-group         = "/ecs/${var.project_name}"
          awslogs-region       = var.aws_region
          awslogs-stream-prefix = "ecs"
        }
      }
      # Health check configuration
      healthCheck = {
        command     = ["CMD-SHELL", "curl -f http://localhost:8080/health || exit 1"]
        interval    = 30
        timeout     = 5
        retries     = 3
        startPeriod = 60
      }
      # Resource limits
      ulimits = [
        {
          name      = "nofile"
          softLimit = 65536
          hardLimit = 65536
        }
      ]
    }
  ])

  # Example task placement constraints
  placement_constraints {
    type       = "memberOf"
    expression = "attribute:ecs.availability-zone in [${join(",", var.availability_zones)}]"
  }

  tags = {
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# Database Configuration
# ---------------------
# RDS PostgreSQL configuration for persistent data storage
# The database is configured for high availability and automatic backups

# RDS Subnet Group for database placement
resource "aws_db_subnet_group" "main" {
  name       = "${var.project_name}-db-subnet-group"
  subnet_ids = aws_subnet.public[*].id

  tags = {
    Name = "${var.project_name}-db-subnet-group"
    Environment = var.environment
  }
}

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
  backup_window          = "03:00-04:00"   # Daily backup window
  maintenance_window     = "Mon:04:00-Mon:05:00"  # Weekly maintenance window
  multi_az               = var.environment == "production"  # Enable Multi-AZ for production
  storage_encrypted      = true            # Enable encryption at rest
  deletion_protection    = var.environment == "production"  # Prevent accidental deletion in production

  # Performance Insights configuration
  performance_insights_enabled = true
  performance_insights_retention_period = 7

  # Enhanced monitoring
  monitoring_interval = var.enable_enhanced_monitoring ? 60 : 0
  monitoring_role_arn = var.enable_enhanced_monitoring ? aws_iam_role.rds_monitoring.arn : null

  # Example parameter group for performance tuning
  parameter_group_name = aws_db_parameter_group.main.name

  tags = {
    Name = "${var.project_name}-db"
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# RDS Parameter Group for performance tuning
resource "aws_db_parameter_group" "main" {
  name   = "${var.project_name}-db-params"
  family = "postgres15"

  parameter {
    name  = "shared_buffers"
    value = "256MB"
  }

  parameter {
    name  = "effective_cache_size"
    value = "768MB"
  }

  parameter {
    name  = "maintenance_work_mem"
    value = "64MB"
  }

  parameter {
    name  = "checkpoint_completion_target"
    value = "0.9"
  }

  parameter {
    name  = "wal_buffers"
    value = "16MB"
  }

  parameter {
    name  = "default_statistics_target"
    value = "100"
  }

  parameter {
    name  = "random_page_cost"
    value = "1.1"
  }

  parameter {
    name  = "effective_io_concurrency"
    value = "200"
  }

  parameter {
    name  = "work_mem"
    value = "2621kB"
  }

  parameter {
    name  = "min_wal_size"
    value = "1GB"
  }

  parameter {
    name  = "max_wal_size"
    value = "4GB"
  }
}

# Load Balancer Configuration
# --------------------------
# Application Load Balancer configuration for distributing traffic
# across multiple application instances

# Application Load Balancer for distributing traffic
resource "aws_lb" "main" {
  name               = "${var.project_name}-alb"
  internal           = false               # Internet-facing
  load_balancer_type = "application"
  security_groups    = [aws_security_group.alb.id]
  subnets           = aws_subnet.public[*].id

  # Enable access logs for monitoring and troubleshooting
  access_logs {
    bucket  = aws_s3_bucket.alb_logs.bucket
    prefix  = "alb"
    enabled = true
  }

  # Enable deletion protection in production
  enable_deletion_protection = var.environment == "production"

  tags = {
    Name = "${var.project_name}-alb"
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# ALB Target Group for ECS tasks
resource "aws_lb_target_group" "app" {
  name        = "${var.project_name}-app-tg"
  port        = 8080
  protocol    = "HTTP"
  vpc_id      = aws_vpc.main.id
  target_type = "ip"

  health_check {
    enabled             = true
    interval            = 30
    path                = "/health"
    port                = "traffic-port"
    protocol            = "HTTP"
    timeout             = 5
    healthy_threshold   = 2
    unhealthy_threshold = 3
    matcher             = "200"
  }

  # Enable stickiness for session persistence
  stickiness {
    type            = "lb_cookie"
    cookie_duration = 86400
    enabled         = var.environment == "production"
  }

  tags = {
    Name = "${var.project_name}-app-tg"
    Environment = var.environment
  }
}

# ALB Listener for HTTP traffic
resource "aws_lb_listener" "http" {
  load_balancer_arn = aws_lb.main.arn
  port              = 80
  protocol          = "HTTP"

  default_action {
    type = "redirect"

    redirect {
      port        = "443"
      protocol    = "HTTPS"
      status_code = "HTTP_301"
    }
  }
}

# ALB Listener for HTTPS traffic
resource "aws_lb_listener" "https" {
  load_balancer_arn = aws_lb.main.arn
  port              = 443
  protocol          = "HTTPS"
  ssl_policy        = "ELBSecurityPolicy-TLS-1-2-2017-01"
  certificate_arn   = aws_acm_certificate.main.arn

  default_action {
    type             = "forward"
    target_group_arn = aws_lb_target_group.app.arn
  }
}

# ECS Service Configuration
# ------------------------
# ECS Service configuration for running and maintaining
# the desired number of application instances

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
    assign_public_ip = true  # Required for Fargate tasks in public subnets
  }

  # Load balancer target group attachment
  load_balancer {
    target_group_arn = aws_lb_target_group.app.arn
    container_name   = "${var.project_name}-app"
    container_port   = 8080
  }

  # Enable service auto-scaling
  capacity_provider_strategy {
    capacity_provider = "FARGATE"
    weight            = 100
  }

  # Configure deployment settings
  deployment_controller {
    type = "ECS"
  }

  # Configure health check grace period
  health_check_grace_period_seconds = 60

  # Example deployment configuration
  deployment_configuration {
    maximum_percent         = 200
    minimum_healthy_percent = 50
  }

  # Example service discovery configuration
  service_registries {
    registry_arn = aws_service_discovery_service.main.arn
  }

  depends_on = [aws_lb_listener.https]
}

# Monitoring Configuration
# -----------------------
# CloudWatch configuration for logging and monitoring

# CloudWatch Log Group for container logs
resource "aws_cloudwatch_log_group" "ecs" {
  name              = "/ecs/${var.project_name}"
  retention_in_days = var.log_retention_days
  kms_key_id        = aws_kms_key.logs.arn # Encrypt logs at rest

  tags = {
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# Example CloudWatch Alarms
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
  alarm_actions      = [aws_sns_topic.alerts.arn]

  dimensions = {
    ClusterName = aws_ecs_cluster.main.name
    ServiceName = aws_ecs_service.main.name
  }
}

resource "aws_cloudwatch_metric_alarm" "memory_utilization" {
  alarm_name          = "${var.project_name}-memory-utilization"
  comparison_operator = "GreaterThanThreshold"
  evaluation_periods  = "2"
  metric_name         = "MemoryUtilization"
  namespace           = "AWS/ECS"
  period             = "300"
  statistic          = "Average"
  threshold          = "80"
  alarm_description  = "Memory utilization exceeds 80%"
  alarm_actions      = [aws_sns_topic.alerts.arn]

  dimensions = {
    ClusterName = aws_ecs_cluster.main.name
    ServiceName = aws_ecs_service.main.name
  }
}

# Security Configuration
# ---------------------
# Security group configurations for different components
# Each security group implements the principle of least privilege

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
    description     = "Allow traffic from ALB"
  }

  # Allow all outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
    description = "Allow all outbound traffic"
  }

  tags = {
    Name = "${var.project_name}-app-sg"
    Environment = var.environment
    ManagedBy = "terraform"
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
    cidr_blocks = var.allowed_cidr_blocks
    description = "Allow HTTP traffic"
  }

  # Allow HTTPS traffic
  ingress {
    from_port   = 443
    to_port     = 443
    protocol    = "tcp"
    cidr_blocks = var.allowed_cidr_blocks
    description = "Allow HTTPS traffic"
  }

  # Allow all outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
    description = "Allow all outbound traffic"
  }

  tags = {
    Name = "${var.project_name}-alb-sg"
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# Security group for RDS database
resource "aws_security_group" "db" {
  name        = "${var.project_name}-db-sg"
  description = "Security group for database"
  vpc_id      = aws_vpc.main.id

  # Allow inbound traffic from application containers only
  ingress {
    from_port       = 5432
    to_port         = 5432
    protocol        = "tcp"
    security_groups = [aws_security_group.app.id]
    description     = "Allow PostgreSQL traffic from app containers"
  }

  # Allow all outbound traffic
  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
    description = "Allow all outbound traffic"
  }

  tags = {
    Name = "${var.project_name}-db-sg"
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# Example IAM roles and policies
resource "aws_iam_role" "ecs_execution_role" {
  name = "${var.project_name}-ecs-execution-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

resource "aws_iam_role_policy_attachment" "ecs_execution_role_policy" {
  role       = aws_iam_role.ecs_execution_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
}

resource "aws_iam_role" "ecs_task_role" {
  name = "${var.project_name}-ecs-task-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "ecs-tasks.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Environment = var.environment
    ManagedBy = "terraform"
  }
}

# Example custom IAM policy for the application
resource "aws_iam_policy" "app_policy" {
  name = "${var.project_name}-app-policy"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "${aws_cloudwatch_log_group.ecs.arn}:*"
      },
      {
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue"
        ]
        Resource = [
          aws_secretsmanager_secret.db_credentials.arn
        ]
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "app_policy_attachment" {
  role       = aws_iam_role.ecs_task_role.name
  policy_arn = aws_iam_policy.app_policy.arn
} 