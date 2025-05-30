#!/bin/bash

# Exit on error
set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MNIST_DIR="${PROJECT_ROOT}/docker/mnist-training"
AWS_REGION="us-east-2"
ENV_FILE="${MNIST_DIR}/.env"

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

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."

    # Check if AWS CLI is installed
    if ! command -v aws &> /dev/null; then
        print_error "AWS CLI is not installed. Please install it first."
        exit 1
    fi

    # Check if Docker is installed
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed. Please install it first."
        exit 1
    fi

    # Check if Docker daemon is running
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running. Please start Docker Desktop and try again."
        exit 1
    fi

    # Check if Python is installed
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is not installed. Please install it first."
        exit 1
    fi

    print_status "All prerequisites are satisfied."
}

# Setup Python virtual environment
setup_python_env() {
    print_status "Setting up Python virtual environment..."

    # Create virtual environment if it doesn't exist
    if [ ! -d "${MNIST_DIR}/venv" ]; then
        python3 -m venv "${MNIST_DIR}/venv"
    fi

    # Activate virtual environment and install requirements
    source "${MNIST_DIR}/venv/bin/activate"
    pip install --upgrade pip
    pip install -r "${MNIST_DIR}/requirements.txt"

    print_status "Python environment setup complete."
}

# Create environment file
create_env_file() {
    print_status "Creating environment file..."

    # Get AWS account ID
    AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
    S3_BUCKET_NAME="contract-management-bucket-bkvww1ps"

    # Create .env file
    cat > "${ENV_FILE}" << EOF
# AWS Configuration
AWS_REGION=${AWS_REGION}
AWS_ACCOUNT_ID=${AWS_ACCOUNT_ID}

# Training Configuration
MODEL_CHECKPOINT_PATH=s3://${S3_BUCKET_NAME}/models/mnist
EPOCHS=10
BATCH_SIZE=32

# ECR Configuration
ECR_REPOSITORY=mnist-training
IMAGE_TAG=latest
EOF

    print_status "Environment file created at ${ENV_FILE}"
}

# Deploy Docker image
deploy_docker() {
    print_status "Deploying Docker image..."

    # Make deploy script executable
    chmod +x "${MNIST_DIR}/deploy_mnist.sh"

    # Run deploy script
    cd "${MNIST_DIR}"
    ./deploy_mnist.sh
}

# Main execution
main() {
    print_status "Starting setup and deployment process..."

    # Check prerequisites
    check_prerequisites

    # Setup Python environment
    setup_python_env

    # Create environment file
    create_env_file

    # Deploy Docker image
    deploy_docker

    print_status "Setup and deployment completed successfully!"
    print_status "You can now run the training using the ECS service."
}

# Run main function
main 