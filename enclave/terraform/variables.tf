variable "aws_region" {
  description = "AWS region to deploy resources"
  type        = string
  default     = "us-east-2"
}

variable "vpc_cidr" {
  description = "CIDR block for the VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "bucket_name" {
  description = "Name of the S3 bucket for encrypted data"
  type        = string
  default     = "mmju_luks"
}

variable "kms_key_alias" {
  description = "Alias for the KMS key"
  type        = string
  default     = "enclave-training-key"
}

variable "key_name" {
  description = "Name of the SSH key pair"
  type        = string
  default     = "enclave-key"
}

variable "instance_type" {
  description = "EC2 instance type"
  type        = string
  default     = "c5.xlarge"
}

variable "root_volume_size" {
  description = "Size of the root volume in GB"
  type        = number
  default     = 50
}

variable "root_volume_type" {
  description = "Type of the root volume"
  type        = string
  default     = "gp3"
}

variable "ami_id" {
  description = "ID of the Nitro Enclaves AMI"
  type        = string
} 