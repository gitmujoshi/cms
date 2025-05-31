variable "security_group_id" {
  description = "Security group ID for the enclave instances"
  type        = string
}

variable "private_subnet_ids" {
  description = "List of private subnet IDs for the enclave instances"
  type        = list(string)
}

variable "container_image" {
  description = "Docker image for the enclave container"
  type        = string
}

variable "ecs_cluster_id" {
  description = "ID of the ECS cluster"
  type        = string
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