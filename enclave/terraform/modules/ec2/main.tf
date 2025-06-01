resource "aws_instance" "enclave" {
  ami           = var.ami_id
  instance_type = var.instance_type
  subnet_id     = var.subnet_id
  key_name      = var.key_name

  vpc_security_group_ids = [var.security_group_id]
  iam_instance_profile   = var.iam_instance_profile
  associate_public_ip_address = true

  root_block_device {
    volume_size = var.root_volume_size
    volume_type = var.root_volume_type
    encrypted   = true
    kms_key_id  = var.kms_key_id
  }

  enclave_options {
    enabled = true
  }

  user_data = <<-EOF
              #!/bin/bash
              
              # Install Docker first (most important)
              amazon-linux-extras install docker -y
              systemctl enable docker
              systemctl start docker
              usermod -a -G docker ec2-user
              
              # Install Nitro Enclaves CLI
              amazon-linux-extras enable aws-nitro-enclaves-cli
              amazon-linux-extras install aws-nitro-enclaves-cli -y
              
              # Set up Nitro Enclaves environment
              mkdir -p /var/log/nitro_enclaves
              mkdir -p /usr/share/nitro_enclaves/blobs
              chown -R ec2-user:ec2-user /var/log/nitro_enclaves
              chown -R ec2-user:ec2-user /usr/share/nitro_enclaves
              chmod 755 /var/log/nitro_enclaves
              chmod 755 /usr/share/nitro_enclaves
              chmod 755 /usr/share/nitro_enclaves/blobs
              
              # Create required files
              touch /usr/share/nitro_enclaves/blobs/cmdline
              chown ec2-user:ec2-user /usr/share/nitro_enclaves/blobs/cmdline
              chmod 644 /usr/share/nitro_enclaves/blobs/cmdline
              
              # Enable and start Nitro Enclaves allocator service
              systemctl enable nitro-enclaves-allocator.service
              systemctl start nitro-enclaves-allocator.service
              
              # Install jq for JSON parsing
              yum install -y jq
              
              # Run system update in background
              yum update -y &
              EOF

  tags = merge(
    {
      Name = "EnclaveInstance"
    },
    var.tags
  )
} 