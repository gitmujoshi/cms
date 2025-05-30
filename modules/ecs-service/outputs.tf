output "task_definition_arn" {
  description = "ARN of the task definition"
  value       = aws_ecs_task_definition.mnist_training.arn
}

output "service_name" {
  description = "Name of the ECS service"
  value       = aws_ecs_service.mnist_training.name
}

output "log_group_name" {
  description = "Name of the CloudWatch log group"
  value       = aws_cloudwatch_log_group.mnist_training.name
} 