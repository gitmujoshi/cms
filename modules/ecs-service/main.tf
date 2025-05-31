resource "aws_ecs_task_definition" "mnist_training" {
  family                   = var.task_family
  network_mode            = "awsvpc"
  requires_compatibilities = ["FARGATE"]
  cpu                     = var.task_cpu
  memory                  = var.task_memory
  execution_role_arn      = var.execution_role_arn
  task_role_arn           = var.task_role_arn

  container_definitions = jsonencode([
    {
      name      = "mnist-training"
      image     = var.container_image
      essential = true
      
      portMappings = [
        {
          containerPort = 80
          hostPort      = 80
          protocol      = "tcp"
        }
      ]
      
      environment = [
        {
          name  = "MODEL_CHECKPOINT_PATH"
          value = "s3://${var.s3_bucket_name}/models/mnist"
        },
        {
          name  = "EPOCHS"
          value = var.training_epochs
        },
        {
          name  = "BATCH_SIZE"
          value = var.batch_size
        }
      ]

      logConfiguration = {
        logDriver = "awslogs"
        options = {
          "awslogs-group"         = "/ecs/${var.task_family}"
          "awslogs-region"        = var.aws_region
          "awslogs-stream-prefix" = "ecs"
        }
      }
    }
  ])

  tags = var.tags
}

resource "aws_ecs_service" "mnist_training" {
  name            = "${var.task_family}-service"
  cluster         = var.ecs_cluster_id
  task_definition = aws_ecs_task_definition.mnist_training.arn
  desired_count   = var.desired_count
  launch_type     = "FARGATE"

  network_configuration {
    subnets          = var.private_subnet_ids
    security_groups  = [var.ecs_security_group_id]
    assign_public_ip = false
  }

  load_balancer {
    target_group_arn = var.target_group_arn
    container_name   = "mnist-training"
    container_port   = 80
  }

  tags = var.tags
}

resource "aws_cloudwatch_log_group" "mnist_training" {
  name              = "/ecs/${var.task_family}"
  retention_in_days = 30
  tags              = var.tags
} 