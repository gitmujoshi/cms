resource "aws_instance" "enclave" {
  ami           = var.ami_id
  instance_type = var.instance_type
  subnet_id     = var.subnet_id
  key_name      = var.key_name

  vpc_security_group_ids = [aws_security_group.main.id]
  iam_instance_profile   = var.iam_instance_profile

  root_block_device {
    volume_size = var.root_volume_size
    volume_type = var.root_volume_type
  }

  enclave_options {
    enabled = true
  }

  user_data = <<-EOF
              #!/bin/bash
              yum update -y
              yum install -y aws-nitro-enclaves-cli aws-nitro-enclaves-cli-devel
              systemctl enable nitro-enclaves-allocator.service
              systemctl start nitro-enclaves-allocator.service
              EOF

  tags = {
    Name = "EnclaveInstance"
  }
}

resource "aws_security_group" "main" {
  name        = "nitro-enclave-sg"
  description = "Security group for Nitro Enclave instance"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = ["0.0.0.0/0"]
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }
} 