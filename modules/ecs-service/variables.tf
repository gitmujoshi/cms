variable "task_family" {
  description = "The family name of the ECS task."
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
  description = "The Docker image to use for the ECS task."
  type        = string
}

variable "execution_role_arn" {
  description = "The ARN of the IAM role for the ECS task execution."
  type        = string
}

variable "task_role_arn" {
  description = "The ARN of the IAM role for the ECS task."
  type        = string
}

variable "ecs_cluster_id" {
  description = "The ID of the ECS cluster."
  type        = string
}

variable "private_subnet_ids" {
  description = "The IDs of the private subnets."
  type        = list(string)
}

variable "ecs_security_group_id" {
  description = "The ID of the security group for the ECS tasks."
  type        = string
}

variable "target_group_arn" {
  description = "The ARN of the target group."
  type        = string
}

variable "s3_bucket_name" {
  description = "The name of the S3 bucket."
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
  description = "The AWS region."
  type        = string
}

variable "tags" {
  description = "A map of tags to assign to the resources."
  type        = map(string)
  default     = {}
} 