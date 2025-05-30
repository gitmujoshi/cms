# Docker Training Environment

This directory contains Docker configurations and training scripts for the MNIST training application.

## Directory Structure

```
docker/
├── README.md
└── mnist-training/
    ├── Dockerfile
    ├── requirements.txt
    ├── train_mnist.py
    └── deploy_mnist.sh
```

## Components

- `mnist-training/`: Directory containing MNIST training application
  - `Dockerfile`: Defines the container environment for MNIST training
  - `requirements.txt`: Python dependencies for the training script
  - `train_mnist.py`: MNIST training script using TensorFlow
  - `deploy_mnist.sh`: Script to build and push the Docker image to ECR

## Usage

1. Navigate to the mnist-training directory:
   ```bash
   cd mnist-training
   ```

2. Build and deploy the Docker image:
   ```bash
   ./deploy_mnist.sh
   ```

## Requirements

- AWS CLI configured with appropriate credentials
- Docker installed and running
- Python 3.9 or later
- Required Python packages (listed in requirements.txt)

## Environment Variables

The training script uses the following environment variables:
- `MODEL_CHECKPOINT_PATH`: S3 path for saving model checkpoints
- `EPOCHS`: Number of training epochs (default: 10)
- `BATCH_SIZE`: Training batch size (default: 32)

# MNIST Training Deployment Guide

This guide provides step-by-step instructions for setting up, building, deploying, and running the MNIST training application in AWS.

## Prerequisites

Before you begin, ensure you have the following installed and configured:

1. **AWS CLI**
   ```bash
   # Install AWS CLI
   brew install awscli  # For macOS
   
   # Configure AWS CLI
   aws configure
   # Enter your AWS Access Key ID
   # Enter your AWS Secret Access Key
   # Enter your default region (us-east-2)
   # Enter your output format (json)
   ```

2. **Docker Desktop**
   - Download and install from [Docker's website](https://www.docker.com/products/docker-desktop)
   - Start Docker Desktop application

3. **Python 3.9 or later**
   ```bash
   # Install Python
   brew install python@3.9  # For macOS
   ```

## Project Structure

```
docker/
├── README.md
├── setup_and_deploy.sh
└── mnist-training/
    ├── Dockerfile
    ├── requirements.txt
    ├── train_mnist.py
    └── deploy_mnist.sh
```

## Step 1: Initial Setup

1. Clone the repository and navigate to the project root:
   ```bash
   cd /path/to/Contract\ Management\ System
   ```

2. Make the setup script executable:
   ```bash
   chmod +x docker/setup_and_deploy.sh
   ```

## Step 2: Build and Deploy

1. Run the setup and deployment script:
   ```bash
   ./docker/setup_and_deploy.sh
   ```

   This script will:
   - Check all prerequisites
   - Set up a Python virtual environment
   - Create environment configuration
   - Build and push the Docker image to ECR

2. Verify the ECR repository:
   ```bash
   aws ecr describe-repositories --repository-names mnist-training
   ```

## Step 3: Infrastructure Setup

1. Navigate to the deployment directory:
   ```bash
   cd deployment
   ```

2. Initialize Terraform:
   ```bash
   terraform init
   ```

3. Review the planned changes:
   ```bash
   terraform plan
   ```

4. Apply the infrastructure:
   ```bash
   terraform apply
   ```

   This will create:
   - VPC with public and private subnets
   - ECS cluster
   - ECR repository
   - S3 bucket for model checkpoints
   - IAM roles and policies
   - Application Load Balancer

## Step 4: Monitor Training

1. View CloudWatch logs:
   ```bash
   aws logs get-log-events \
       --log-group-name /ecs/mnist-training \
       --log-stream-name ecs/mnist-training/latest
   ```

2. Check ECS service status:
   ```bash
   aws ecs describe-services \
       --cluster contract-management-cluster \
       --services mnist-training-service
   ```

3. Monitor S3 bucket for model checkpoints:
   ```bash
   aws s3 ls s3://contract-management-bucket-bkvww1ps/models/mnist/
   ```

## Step 5: Cleanup (Optional)

To clean up all resources when you're done:

1. Stop the ECS service:
   ```bash
   aws ecs update-service \
       --cluster contract-management-cluster \
       --service mnist-training-service \
       --desired-count 0
   ```

2. Destroy Terraform infrastructure:
   ```bash
   cd deployment
   terraform destroy
   ```

3. Delete ECR repository:
   ```bash
   aws ecr delete-repository \
       --repository-name mnist-training \
       --force
   ```

## Troubleshooting

### Common Issues

1. **Docker daemon not running**
   - Start Docker Desktop
   - Wait for the whale icon to appear in the menu bar

2. **AWS credentials not configured**
   - Run `aws configure` and enter your credentials
   - Verify with `aws sts get-caller-identity`

3. **ECR repository already exists**
   - This is normal, the script will handle it
   - No action needed

4. **Permission denied errors**
   - Ensure your AWS IAM user has necessary permissions
   - Check IAM policies for ECR, ECS, and S3 access

### Logs and Debugging

1. **View ECS task logs**
   ```bash
   aws logs get-log-events \
       --log-group-name /ecs/mnist-training \
       --log-stream-name ecs/mnist-training/latest
   ```

2. **Check ECS task status**
   ```bash
   aws ecs describe-tasks \
       --cluster contract-management-cluster \
       --tasks $(aws ecs list-tasks --cluster contract-management-cluster --query 'taskArns[]' --output text)
   ```

## Additional Resources

- [AWS ECS Documentation](https://docs.aws.amazon.com/ecs/)
- [Docker Documentation](https://docs.docker.com/)
- [Terraform Documentation](https://www.terraform.io/docs)
- [TensorFlow MNIST Tutorial](https://www.tensorflow.org/tutorials/keras/classification) 