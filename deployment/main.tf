terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
  required_version = ">= 1.2.0"
}

provider "aws" {
  region = var.aws_region
}

# VPC Module
module "vpc" {
  source = "./modules/vpc"

  vpc_name             = "${var.project_name}-vpc"
  vpc_cidr            = var.vpc_cidr
  availability_zones  = var.availability_zones
  private_subnet_cidrs = var.private_subnet_cidrs
  public_subnet_cidrs  = var.public_subnet_cidrs
  tags                = var.tags
}

# ECS Module
module "ecs" {
  source = "./modules/ecs"

  cluster_name = "${var.project_name}-cluster"
  vpc_id      = module.vpc.vpc_id
  tags        = var.tags
}

# RDS Module
module "rds" {
  source = "./modules/rds"

  db_identifier              = "${var.project_name}-db"
  db_instance_class         = var.db_instance_class
  allocated_storage         = 20
  db_name                   = var.db_name
  db_username               = var.db_username
  vpc_id                    = module.vpc.vpc_id
  subnet_ids                = module.vpc.private_subnets
  ecs_tasks_security_group_id = module.ecs.ecs_tasks_security_group_id
  tags                      = var.tags
}

# ALB Security Group
resource "aws_security_group" "alb" {
  name        = "${var.project_name}-alb-sg"
  description = "Security group for ALB"
  vpc_id      = module.vpc.vpc_id

  ingress {
    from_port   = 80
    to_port     = 80
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = var.tags
}

# S3 Module
module "s3" {
  source         = "./modules/s3"
  bucket_name    = "${var.project_name}-bucket"
  force_destroy  = true
  versioning_enabled = true
  tags           = var.tags
}

# IAM Module (ECS Task Execution Role)
module "ecs_task_iam" {
  source    = "./modules/iam"
  role_name = "${var.project_name}-ecs-task-execution-role"
  policy_arn = "arn:aws:iam::aws:policy/service-role/AmazonECSTaskExecutionRolePolicy"
  tags      = var.tags
}

# ALB Module
module "alb" {
  source              = "./modules/alb"
  alb_name            = "${var.project_name}-alb"
  security_group_ids  = [aws_security_group.alb.id]
  subnet_ids          = module.vpc.public_subnets
  target_group_name   = "${var.project_name}-tg"
  target_group_port   = 80
  vpc_id              = module.vpc.vpc_id
  tags                = var.tags
}

# ECR Repository for MNIST training
data "aws_ecr_repository" "mnist" {
  name = "mnist-training"
}

module "ecs_service" {
  source = "../modules/ecs-service"

  task_family           = "mnist-training"
  container_image       = "${data.aws_ecr_repository.mnist.repository_url}:latest"
  execution_role_arn    = module.ecs_task_iam.role_arn
  task_role_arn         = module.ecs_task_iam.role_arn
  ecs_cluster_id        = module.ecs.cluster_id
  private_subnet_ids    = module.vpc.private_subnet_ids
  ecs_security_group_id = module.ecs.ecs_tasks_security_group_id
  target_group_arn      = module.alb.target_group_arn
  s3_bucket_name        = module.s3.bucket_name
  aws_region            = var.aws_region
  tags                  = var.tags
} 