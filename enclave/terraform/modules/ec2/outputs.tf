output "instance_id" {
  description = "The ID of the EC2 instance"
  value       = aws_instance.enclave.id
}

output "instance_public_ip" {
  description = "The public IP address of the EC2 instance"
  value       = aws_instance.enclave.public_ip
}

output "instance_private_ip" {
  value = aws_instance.enclave.private_ip
} 