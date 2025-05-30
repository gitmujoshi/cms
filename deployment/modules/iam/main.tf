resource "aws_iam_role" "ecs_task_execution" {
  name = var.role_name
  assume_role_policy = data.aws_iam_policy_document.assume_role_policy.json
  tags = var.tags
}

data "aws_iam_policy_document" "assume_role_policy" {
  statement {
    actions = ["sts:AssumeRole"]
    principals {
      type        = "Service"
      identifiers = ["ecs-tasks.amazonaws.com"]
    }
  }
}

resource "aws_iam_role_policy_attachment" "ecs_task_execution" {
  role       = aws_iam_role.ecs_task_execution.name
  policy_arn = var.policy_arn
} 