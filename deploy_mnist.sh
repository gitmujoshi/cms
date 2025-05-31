#!/bin/bash

# Exit on error
set -e

# Configuration
AWS_REGION="us-east-1"
ECR_REPOSITORY="mnist-training"
IMAGE_TAG="latest"
LOG_GROUP="/ecs/mnist-training"
OUTPUT_DIR="docs/training_logs"

# Get AWS account ID
AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

# Create ECR repository if it doesn't exist
aws ecr describe-repositories --repository-names ${ECR_REPOSITORY} || \
    aws ecr create-repository --repository-name ${ECR_REPOSITORY}

# Get ECR login token and login to Docker
aws ecr get-login-password --region ${AWS_REGION} | \
    docker login --username AWS --password-stdin ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com

# Build the Docker image
echo "Building Docker image..."
docker build -t ${ECR_REPOSITORY}:${IMAGE_TAG} .

# Tag the image for ECR
docker tag ${ECR_REPOSITORY}:${IMAGE_TAG} ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ECR_REPOSITORY}:${IMAGE_TAG}

# Push the image to ECR
echo "Pushing image to ECR..."
docker push ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ECR_REPOSITORY}:${IMAGE_TAG}

echo "Image successfully pushed to ECR!"
echo "ECR Repository URL: ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ECR_REPOSITORY}:${IMAGE_TAG}"

# Function to download and format logs
download_logs() {
    local log_stream=$1
    local timestamp=$(date +%Y%m%d_%H%M%S)
    
    # Create output directory if it doesn't exist
    mkdir -p "$OUTPUT_DIR"
    
    echo "Downloading logs from CloudWatch..."
    aws logs get-log-events \
        --log-group-name "$LOG_GROUP" \
        --log-stream-name "$log_stream" \
        --region "$AWS_REGION" \
        --no-cli-pager > "$OUTPUT_DIR/training_logs_$timestamp.json"
    
    # Convert JSON to readable format
    echo "Converting logs to readable format..."
    cat "$OUTPUT_DIR/training_logs_$timestamp.json" | jq -r '.events[] | .message' > "$OUTPUT_DIR/training_logs_$timestamp.txt"
    
    # Copy the formatted logs to the main training_logs.txt
    cp "$OUTPUT_DIR/training_logs_$timestamp.txt" training_logs.txt
    
    echo "Logs have been downloaded and formatted successfully!"
    echo "JSON logs saved to: $OUTPUT_DIR/training_logs_$timestamp.json"
    echo "Formatted logs saved to: $OUTPUT_DIR/training_logs_$timestamp.txt"
    echo "Main training logs updated at: training_logs.txt"
}

# Wait for training to complete and download logs
echo "Waiting for training to complete..."
sleep 300  # Wait for 5 minutes to allow training to start

# Get the latest log stream
LATEST_LOG_STREAM=$(aws logs describe-log-streams \
    --log-group-name "$LOG_GROUP" \
    --order-by LastEventTime \
    --descending \
    --limit 1 \
    --query 'logStreams[0].logStreamName' \
    --output text)

if [ "$LATEST_LOG_STREAM" != "None" ]; then
    echo "Found log stream: $LATEST_LOG_STREAM"
    download_logs "$LATEST_LOG_STREAM"
else
    echo "No log streams found. Training may not have started yet."
fi 