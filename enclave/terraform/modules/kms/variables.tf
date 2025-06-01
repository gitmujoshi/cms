variable "key_alias" {
  description = "Alias for the KMS key"
  type        = string
  default     = "enclave-training-key"
}

variable "ec2_role_arn" {
  description = "ARN of the EC2 instance role"
  type        = string
} 