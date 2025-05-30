variable "task_family" {
  description = "Family name for the ECS task definition"
  type        = string
}

variable "task_cpu" {
  description = "CPU units for the task"
  type        = number
  default     = 1024
}

variable "task_memory" {
  description = "Memory for the task in MB"
  type        = number
  default     = 2048
}

variable "container_image" {
  description = "Docker image for the MNIST training container"
  type        = string
}

variable "execution_role_arn" {
  description = "ARN of the ECS task execution role"
  type        = string
}

variable "task_role_arn" {
  description = "ARN of the ECS task role"
  type        = string
}

variable "ecs_cluster_id" {
  description = "ID of the ECS cluster"
  type        = string
}

variable "private_subnet_ids" {
  description = "List of private subnet IDs for the ECS service"
  type        = list(string)
}

variable "ecs_security_group_id" {
  description = "Security group ID for the ECS service"
  type        = string
}

variable "target_group_arn" {
  description = "ARN of the target group for the service"
  type        = string
}

variable "s3_bucket_name" {
  description = "Name of the S3 bucket for model checkpoints"
  type        = string
}

variable "training_epochs" {
  description = "Number of training epochs"
  type        = string
  default     = "10"
}

variable "batch_size" {
  description = "Training batch size"
  type        = string
  default     = "32"
}

variable "desired_count" {
  description = "Desired number of tasks"
  type        = number
  default     = 1
}

variable "aws_region" {
  description = "AWS region"
  type        = string
}

variable "tags" {
  description = "A map of tags to add to all resources"
  type        = map(string)
  default     = {}
} 