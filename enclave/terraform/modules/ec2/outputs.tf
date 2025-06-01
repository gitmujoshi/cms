output "instance_id" {
  value = aws_instance.enclave.id
}

output "instance_public_ip" {
  value = aws_instance.enclave.public_ip
} 