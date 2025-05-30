output "db_instance_endpoint" {
  description = "The connection endpoint for the RDS instance"
  value       = module.db.db_instance_endpoint
}

output "db_instance_port" {
  description = "The port the RDS instance is listening on"
  value       = module.db.db_instance_port
}

output "db_instance_name" {
  description = "The name of the RDS instance"
  value       = module.db.db_instance_name
}

output "rds_security_group_id" {
  description = "ID of the security group for RDS"
  value       = aws_security_group.rds.id
} 