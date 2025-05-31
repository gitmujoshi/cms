# Nitro Enclave Configuration Module

# EC2 Launch Template for Enclave-enabled instances
resource "aws_launch_template" "enclave" {
  name_prefix   = "enclave-"
  image_id      = data.aws_ami.enclave.id
  instance_type = "m6i.xlarge"  # Enclave-enabled instance type

  network_interfaces {
    associate_public_ip_address = false
    security_groups            = [var.security_group_id]
  }

  user_data = base64encode(<<-EOF
              #!/bin/bash
              yum update -y
              yum install -y aws-nitro-enclaves-cli aws-nitro-enclaves-cli-devel
              systemctl enable nitro-enclaves-allocator.service
              systemctl start nitro-enclaves-allocator.service
              EOF
  )

  tag_specifications {
    resource_type = "instance"
    tags = {
      Name = "enclave-instance"
    }
  }
}

# ECS Capacity Provider for Enclave-enabled instances
resource "aws_ecs_capacity_provider" "enclave" {
  name = "enclave-capacity-provider"

  auto_scaling_group_provider {
    auto_scaling_group_arn = aws_autoscaling_group.enclave.arn
    managed_scaling {
      maximum_scaling_step_size = 1000
      minimum_scaling_step_size = 1
      status                    = "ENABLED"
      target_capacity           = 100
    }
  }
}

# Auto Scaling Group for Enclave instances
resource "aws_autoscaling_group" "enclave" {
  name                = "enclave-asg"
  vpc_zone_identifier = var.private_subnet_ids
  desired_capacity    = 1
  max_size           = 3
  min_size           = 1

  launch_template {
    id      = aws_launch_template.enclave.id
    version = "$Latest"
  }

  tag {
    key                 = "AmazonECSManaged"
    value              = true
    propagate_at_launch = true
  }
}

# ECS Task Definition for Enclave-enabled tasks
resource "aws_ecs_task_definition" "enclave" {
  family                   = "enclave-task"
  requires_compatibilities = ["EC2"]
  network_mode            = "awsvpc"
  cpu                     = 2048  # 2 vCPU
  memory                  = 4096  # 4 GB

  container_definitions = jsonencode([
    {
      name  = "enclave-container"
      image = var.container_image
      essential = true

      environment = [
        {
          name  = "ENCLAVE_CPU_COUNT"
          value = "2"
        },
        {
          name  = "ENCLAVE_MEMORY_MIB"
          value = "2048"
        }
      ]

      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = "/ecs/enclave"
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "ecs"
        }
      }
    }
  ])

  runtime_platform {
    operating_system_family = "LINUX"
    cpu_architecture       = "X86_64"
  }

  tags = var.tags
}

# ECS Service for Enclave tasks
resource "aws_ecs_service" "enclave" {
  name            = "enclave-service"
  cluster         = var.ecs_cluster_id
  task_definition = aws_ecs_task_definition.enclave.arn
  desired_count   = 1
  launch_type     = "EC2"

  network_configuration {
    subnets          = var.private_subnet_ids
    security_groups  = [var.security_group_id]
    assign_public_ip = false
  }

  capacity_provider_strategy {
    capacity_provider = aws_ecs_capacity_provider.enclave.name
    weight           = 100
  }

  tags = var.tags
}

# Data source for Enclave-enabled AMI
data "aws_ami" "enclave" {
  most_recent = true
  owners      = ["amazon"]

  filter {
    name   = "name"
    values = ["amzn2-ami-hvm-*-x86_64-gp2"]
  }

  filter {
    name   = "virtualization-type"
    values = ["hvm"]
  }
} 