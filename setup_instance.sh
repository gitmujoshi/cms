#!/bin/bash
set -e

# Install Docker
sudo yum update -y
sudo yum install -y docker
sudo systemctl start docker
sudo systemctl enable docker
sudo usermod -a -G docker ec2-user

# Install Nitro Enclaves CLI
sudo yum install -y aws-nitro-enclaves-cli-dev

# Enable and start Nitro Enclaves
sudo systemctl enable nitro-enclaves-allocator.service
sudo systemctl start nitro-enclaves-allocator.service

# Build and run the enclave
nitro-cli build-enclave --docker-uri ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ENCLAVE_IMAGE_NAME}:${ENCLAVE_IMAGE_TAG} --output-file mnist-training.eif
nitro-cli run-enclave --eif-path mnist-training.eif --cpu-count 2 --memory 4096
