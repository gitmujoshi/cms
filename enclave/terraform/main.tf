terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.0"
    }
    tls = {
      source  = "hashicorp/tls"
      version = "~> 4.0"
    }
  }
}

provider "aws" {
  region = var.aws_region
}

# Generate SSH key pair
resource "tls_private_key" "enclave_key" {
  algorithm = "RSA"
  rsa_bits  = 4096
}

# Save private key to file
resource "local_file" "private_key" {
  content         = tls_private_key.enclave_key.private_key_pem
  filename        = "${path.module}/enclave-key"
  file_permission = "0600"
}

# Save public key to file
resource "local_file" "public_key" {
  content         = tls_private_key.enclave_key.public_key_openssh
  filename        = "${path.module}/enclave-key.pub"
  file_permission = "0644"
}

# Create AWS key pair
resource "aws_key_pair" "enclave_key" {
  key_name   = var.key_name
  public_key = tls_private_key.enclave_key.public_key_openssh
}

# VPC and Networking
module "vpc" {
  source = "./modules/vpc"
  vpc_cidr = var.vpc_cidr
}

# S3 Bucket for encrypted data
module "s3" {
  source = "./modules/s3"
  bucket_name = var.bucket_name
}

# IAM roles and policies
module "iam" {
  source = "./modules/iam"
  s3_bucket_arn = module.s3.bucket_arn
  kms_key_arn = module.kms.key_arn
}

# KMS for encryption
module "kms" {
  source = "./modules/kms"
  key_alias = var.kms_key_alias
  ec2_role_arn = module.iam.role_arn
}

# EC2 Instance with Nitro Enclaves
module "ec2" {
  source = "./modules/ec2"
  vpc_id = module.vpc.vpc_id
  subnet_id = module.vpc.subnet_id
  key_name = aws_key_pair.enclave_key.key_name
  bucket_name = module.s3.bucket_name
  kms_key_id = module.kms.key_id
  iam_instance_profile = module.iam.instance_profile_name
  instance_type = var.instance_type
  root_volume_size = var.root_volume_size
  root_volume_type = var.root_volume_type
  ami_id = var.ami_id
} 