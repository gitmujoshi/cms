variable "role_name" {
  description = "The name of the IAM role."
  type        = string
}

variable "policy_arn" {
  description = "The ARN of the policy to attach."
  type        = string
}

variable "tags" {
  description = "A map of tags to assign to the role."
  type        = map(string)
  default     = {}
} 