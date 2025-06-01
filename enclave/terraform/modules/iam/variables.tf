variable "s3_bucket_arn" {
  description = "The ARN of the S3 bucket"
  type        = string
}

variable "kms_key_arn" {
  description = "The ARN of the KMS key"
  type        = string
}

variable "tags" {
  description = "A map of tags to assign to the role"
  type        = map(string)
  default     = {}
} 