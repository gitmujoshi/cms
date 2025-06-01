output "instance_public_ip" {
  description = "Public IP address of the EC2 instance"
  value       = module.ec2.instance_public_ip
}

output "s3_bucket" {
  description = "Name of the S3 bucket"
  value       = module.s3.bucket_name
}

output "kms_key_id" {
  description = "ID of the KMS key"
  value       = module.kms.key_id
  sensitive   = true
} 