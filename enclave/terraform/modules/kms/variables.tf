variable "key_alias" {
  description = "Alias for the KMS key"
  type        = string
  default     = "enclave-training-key"
}

variable "ec2_role_arn" {
  description = "ARN of the EC2 instance role"
  type        = string
}

variable "tags" {
  description = "A map of tags to assign to the KMS key"
  type        = map(string)
  default     = {}
} 