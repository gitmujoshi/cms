output "role_arn" {
  value = aws_iam_role.enclave.arn
}

output "instance_profile_name" {
  value = aws_iam_instance_profile.enclave.name
} 