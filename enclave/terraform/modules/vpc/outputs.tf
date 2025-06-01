output "vpc_id" {
  value = aws_vpc.main.id
}

output "subnet_id" {
  value = aws_subnet.main.id
}

output "security_group_id" {
  value = length(data.aws_security_group.existing) > 0 ? data.aws_security_group.existing[0].id : aws_security_group.main[0].id
} 