variable "alb_name" {
  description = "The name of the Application Load Balancer."
  type        = string
}

variable "security_group_ids" {
  description = "A list of security group IDs to assign to the ALB."
  type        = list(string)
}

variable "subnet_ids" {
  description = "A list of subnet IDs to launch the ALB in."
  type        = list(string)
}

variable "target_group_name" {
  description = "The name of the target group."
  type        = string
}

variable "target_group_port" {
  description = "The port for the target group."
  type        = number
}

variable "vpc_id" {
  description = "The VPC ID for the target group."
  type        = string
}

variable "tags" {
  description = "A map of tags to assign to resources."
  type        = map(string)
  default     = {}
} 