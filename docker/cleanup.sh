#!/bin/bash

# Exit on error
set -e

# Configuration
AWS_REGION="us-east-2"
ECR_REPOSITORY="mnist-training"
S3_BUCKET="contract-management-bucket-bkvww1ps"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status messages
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if AWS CLI is installed and configured
check_aws_cli() {
    print_status "Checking AWS CLI configuration..."
    
    if ! command -v aws &> /dev/null; then
        print_error "AWS CLI is not installed. Please install it first."
        exit 1
    fi

    # Check if AWS credentials are configured
    if ! aws sts get-caller-identity &> /dev/null; then
        print_error "AWS credentials are not configured. Please run 'aws configure' first."
        exit 1
    fi
}

# Delete ECR repository
delete_ecr_repository() {
    print_status "Deleting ECR repository: ${ECR_REPOSITORY}"
    
    # Check if repository exists
    if aws ecr describe-repositories --repository-names ${ECR_REPOSITORY} &> /dev/null; then
        # Delete all images in the repository
        print_status "Deleting all images in the repository..."
        aws ecr batch-delete-image \
            --repository-name ${ECR_REPOSITORY} \
            --image-ids "$(aws ecr list-images --repository-name ${ECR_REPOSITORY} --query 'imageIds[*]' --output json)" \
            || true

        # Delete the repository
        aws ecr delete-repository \
            --repository-name ${ECR_REPOSITORY} \
            --force
        print_status "ECR repository deleted successfully."
    else
        print_warning "ECR repository ${ECR_REPOSITORY} does not exist."
    fi
}

# Clean up S3 bucket contents
cleanup_s3_bucket() {
    print_status "Cleaning up S3 bucket: ${S3_BUCKET}"
    
    # Check if bucket exists
    if aws s3 ls "s3://${S3_BUCKET}" &> /dev/null; then
        # Delete all objects in the models directory
        print_status "Deleting model checkpoints..."
        aws s3 rm "s3://${S3_BUCKET}/models/mnist/" --recursive || true
        print_status "S3 bucket contents cleaned up successfully."
    else
        print_warning "S3 bucket ${S3_BUCKET} does not exist."
    fi
}

# Clean up local files
cleanup_local_files() {
    print_status "Cleaning up local files..."
    
    # Remove virtual environment
    if [ -d "mnist-training/venv" ]; then
        print_status "Removing virtual environment..."
        rm -rf mnist-training/venv
    fi

    # Remove .env file
    if [ -f "mnist-training/.env" ]; then
        print_status "Removing .env file..."
        rm mnist-training/.env
    fi

    print_status "Local files cleaned up successfully."
}

# Main execution
main() {
    print_status "Starting cleanup process..."

    # Check AWS CLI
    check_aws_cli

    # Delete ECR repository
    delete_ecr_repository

    # Clean up S3 bucket
    cleanup_s3_bucket

    # Clean up local files
    cleanup_local_files

    print_status "Cleanup completed successfully!"
}

# Run main function
main 