#!/bin/bash

# Exit on error
set -e

# Set AWS region and account
AWS_REGION="us-east-2"
AWS_ACCOUNT_ID="124355660528"
REPOSITORY_NAME="mnist-training"
IMAGE_TAG="latest"
LOG_GROUP="/ecs/mnist-training"
OUTPUT_DIR="../docs/training_logs"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to check if Docker daemon is running
check_docker_daemon() {
    echo -e "${GREEN}Checking Docker daemon...${NC}"
    if ! docker info >/dev/null 2>&1; then
        echo -e "${YELLOW}Docker daemon is not running. Attempting to start Docker Desktop...${NC}"
        
        # Try to start Docker Desktop
        if [[ "$OSTYPE" == "darwin"* ]]; then
            echo -e "${GREEN}Starting Docker Desktop...${NC}"
            open -a Docker
            
            # Wait for Docker to start (up to 60 seconds)
            echo -e "${YELLOW}Waiting for Docker to start...${NC}"
            for i in {1..60}; do
                if docker info >/dev/null 2>&1; then
                    echo -e "${GREEN}Docker Desktop started successfully!${NC}"
                    return 0
                fi
                echo -n "."
                sleep 1
            done
            
            echo -e "\n${RED}Error: Docker Desktop failed to start within 60 seconds${NC}"
            echo -e "${YELLOW}Please start Docker Desktop manually and try again.${NC}"
            echo -e "${YELLOW}You can start Docker Desktop by:${NC}"
            echo -e "${YELLOW}1. Opening Docker Desktop application${NC}"
            echo -e "${YELLOW}2. Waiting for the Docker icon in the menu bar to show it's running${NC}"
            echo -e "${YELLOW}3. Then run this script again${NC}"
            exit 1
        else
            echo -e "${RED}Error: Docker daemon is not running${NC}"
            echo -e "${YELLOW}Please start Docker Desktop and try again.${NC}"
            exit 1
        fi
    fi
}

# Function to check if AWS CLI is working
check_aws_cli() {
    echo -e "${GREEN}Checking AWS CLI...${NC}"
    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        echo -e "${RED}Error: AWS CLI is not configured properly${NC}"
        exit 1
    fi
}

# Function to check if ECR repository exists
check_ecr_repo() {
    echo -e "${GREEN}Checking ECR repository...${NC}"
    if ! aws ecr describe-repositories --repository-names ${REPOSITORY_NAME} --region ${AWS_REGION} >/dev/null 2>&1; then
        echo -e "${GREEN}Creating ECR repository...${NC}"
        aws ecr create-repository --repository-name ${REPOSITORY_NAME} --region ${AWS_REGION} >/dev/null 2>&1
    fi
}

# Function to check deployment status
check_deployment_status() {
    local service_name="mnist-training-service"
    local cluster_name="contract-management-cluster"
    
    echo -e "${GREEN}Waiting for deployment to complete...${NC}"
    while true; do
        status=$(aws ecs describe-services --cluster ${cluster_name} --services ${service_name} --region ${AWS_REGION} --query 'services[0].deployments[0].status' --output text --no-cli-pager)
        if [ "$status" = "PRIMARY" ]; then
            echo -e "${GREEN}Deployment completed successfully!${NC}"
            echo -e "${GREEN}You can monitor the service in the AWS ECS console:${NC}"
            echo -e "${GREEN}https://${AWS_REGION}.console.aws.amazon.com/ecs/home?region=${AWS_REGION}#/clusters/${cluster_name}/services/${service_name}/details${NC}"
            break
        fi
        echo -e "${GREEN}Deployment in progress...${NC}"
        sleep 10
    done
}

# Function to generate training report
generate_training_report() {
    local log_file=$1
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local report_file="$OUTPUT_DIR/training_report_$timestamp.txt"
    local md_report_file="$OUTPUT_DIR/training_report_$timestamp.md"
    
    echo -e "${GREEN}Generating training report...${NC}"
    
    # Create report header
    echo "MNIST Training Report" > "$report_file"
    echo "Generated on: $(date)" >> "$report_file"
    echo "==========================================" >> "$report_file"
    echo "" >> "$report_file"
    
    # Extract and summarize key metrics
    echo "Training Summary:" >> "$report_file"
    echo "----------------" >> "$report_file"
    
    # Extract final accuracy from the last epoch
    final_accuracy=$(grep -o "accuracy: [0-9.]*" "$log_file" | tail -n 1)
    echo "Final Accuracy: $final_accuracy" >> "$report_file"
    
    # Extract final loss
    final_loss=$(grep -o "loss: [0-9.]*" "$log_file" | tail -n 1)
    echo "Final Loss: $final_loss" >> "$report_file"
    
    # Calculate training speed
    total_steps=$(grep -o "\[[0-9]*/[0-9]*\]" "$log_file" | tail -n 1 | grep -o "[0-9]*" | head -n 1)
    total_time=$(grep -o "[0-9]*ms/step" "$log_file" | tail -n 1 | grep -o "[0-9]*")
    if [ ! -z "$total_steps" ] && [ ! -z "$total_time" ]; then
        echo "Training Speed: $total_time ms/step" >> "$report_file"
        echo "Total Steps: $total_steps" >> "$report_file"
    fi
    
    # Extract training duration
    start_time=$(grep "Epoch 1/5" "$log_file" | head -n 1 | cut -d' ' -f1,2)
    end_time=$(grep "Epoch [0-9]/5" "$log_file" | tail -n 1 | cut -d' ' -f1,2)
    if [ ! -z "$start_time" ] && [ ! -z "$end_time" ]; then
        echo "Training Duration: $start_time to $end_time" >> "$report_file"
    fi
    
    # Extract system information
    echo "" >> "$report_file"
    echo "System Information:" >> "$report_file"
    echo "-----------------" >> "$report_file"
    grep "TensorFlow binary is optimized" "$log_file" >> "$report_file" 2>/dev/null || true
    grep "GPU will not be used" "$log_file" >> "$report_file" 2>/dev/null || true
    
    # Extract memory usage
    memory_warning=$(grep "exceeds [0-9]*% of free system memory" "$log_file" 2>/dev/null || true)
    if [ ! -z "$memory_warning" ]; then
        echo "Memory Usage: $memory_warning" >> "$report_file"
    fi
    
    # Extract training progress
    echo "" >> "$report_file"
    echo "Training Progress:" >> "$report_file"
    echo "----------------" >> "$report_file"
    
    # Get accuracy progression with timestamps
    echo "Accuracy Progression:" >> "$report_file"
    step=0
    grep -o "accuracy: [0-9.]*" "$log_file" | awk 'NR % 50 == 0' | while read -r line; do
        echo "Step $step: $line" >> "$report_file"
        step=$((step + 50))
    done
    
    # Get loss progression with timestamps
    echo "" >> "$report_file"
    echo "Loss Progression:" >> "$report_file"
    step=0
    grep -o "loss: [0-9.]*" "$log_file" | awk 'NR % 50 == 0' | while read -r line; do
        echo "Step $step: $line" >> "$report_file"
        step=$((step + 50))
    done
    
    # Calculate convergence metrics
    echo "" >> "$report_file"
    echo "Convergence Analysis:" >> "$report_file"
    echo "-------------------" >> "$report_file"
    
    # Get accuracy at different points
    initial_accuracy=$(grep -o "accuracy: [0-9.]*" "$log_file" | head -n 1)
    mid_accuracy=$(grep -o "accuracy: [0-9.]*" "$log_file" | awk 'NR % 2 == 0' | head -n 1)
    final_accuracy=$(grep -o "accuracy: [0-9.]*" "$log_file" | tail -n 1)
    
    echo "Initial Accuracy: $initial_accuracy" >> "$report_file"
    echo "Mid Training Accuracy: $mid_accuracy" >> "$report_file"
    echo "Final Accuracy: $final_accuracy" >> "$report_file"
    
    # Extract any errors or warnings
    echo "" >> "$report_file"
    echo "Errors and Warnings:" >> "$report_file"
    echo "------------------" >> "$report_file"
    grep -i "error\|warning\|exception" "$log_file" 2>/dev/null | grep -v "GPU will not be used" >> "$report_file" || true
    
    echo -e "${GREEN}Training report generated: $report_file${NC}"
    
    # Create markdown version with additional formatting
    {
        echo "# MNIST Training Report"
        echo "Generated on: $(date)"
        echo ""
        echo "## Training Summary"
        echo ""
        echo "| Metric | Value |"
        echo "|--------|-------|"
        echo "| Final Accuracy | $final_accuracy |"
        echo "| Final Loss | $final_loss |"
        if [ ! -z "$total_steps" ] && [ ! -z "$total_time" ]; then
            echo "| Training Speed | $total_time ms/step |"
            echo "| Total Steps | $total_steps |"
        fi
        echo "| Training Duration | $start_time to $end_time |"
        echo ""
        echo "## System Information"
        echo ""
        echo "```"
        grep "TensorFlow binary is optimized" "$log_file" 2>/dev/null || true
        grep "GPU will not be used" "$log_file" 2>/dev/null || true
        if [ ! -z "$memory_warning" ]; then
            echo "$memory_warning"
        fi
        echo "```"
        echo ""
        echo "## Training Progress"
        echo ""
        echo "### Accuracy Progression"
        echo ""
        echo "| Step | Accuracy |"
        echo "|------|----------|"
        step=0
        grep -o "accuracy: [0-9.]*" "$log_file" | awk 'NR % 50 == 0' | while read -r line; do
            echo "| $step | $line |"
            step=$((step + 50))
        done
        echo ""
        echo "### Loss Progression"
        echo ""
        echo "| Step | Loss |"
        echo "|------|------|"
        step=0
        grep -o "loss: [0-9.]*" "$log_file" | awk 'NR % 50 == 0' | while read -r line; do
            echo "| $step | $line |"
            step=$((step + 50))
        done
        echo ""
        echo "## Convergence Analysis"
        echo ""
        echo "| Stage | Accuracy |"
        echo "|-------|----------|"
        echo "| Initial | $initial_accuracy |"
        echo "| Mid Training | $mid_accuracy |"
        echo "| Final | $final_accuracy |"
        echo ""
        echo "## Errors and Warnings"
        echo ""
        echo "```"
        grep -i "error\|warning\|exception" "$log_file" 2>/dev/null | grep -v "GPU will not be used" || true
        echo "```"
    } > "$md_report_file"
    
    echo -e "${GREEN}Markdown report generated: $md_report_file${NC}"
}

# Function to download and format logs
download_logs() {
    local log_stream=$1
    local timestamp=$(date +%Y%m%d_%H%M%S)
    
    echo -e "${GREEN}Creating output directory...${NC}"
    mkdir -p "$OUTPUT_DIR"
    
    echo -e "${GREEN}Downloading logs from CloudWatch...${NC}"
    aws logs get-log-events \
        --log-group-name "$LOG_GROUP" \
        --log-stream-name "$log_stream" \
        --region "$AWS_REGION" \
        --no-cli-pager > "$OUTPUT_DIR/training_logs_$timestamp.json"
    
    echo -e "${GREEN}Converting logs to readable format...${NC}"
    cat "$OUTPUT_DIR/training_logs_$timestamp.json" | jq -r '.events[] | .message' > "$OUTPUT_DIR/training_logs_$timestamp.txt"
    
    echo -e "${GREEN}Updating main training logs...${NC}"
    cp "$OUTPUT_DIR/training_logs_$timestamp.txt" "../training_logs.txt"
    
    # Generate training report
    generate_training_report "$OUTPUT_DIR/training_logs_$timestamp.txt"
    
    echo -e "${GREEN}Logs have been downloaded and formatted successfully!${NC}"
    echo -e "${GREEN}JSON logs saved to: $OUTPUT_DIR/training_logs_$timestamp.json${NC}"
    echo -e "${GREEN}Formatted logs saved to: $OUTPUT_DIR/training_logs_$timestamp.txt${NC}"
    echo -e "${GREEN}Main training logs updated at: ../training_logs.txt${NC}"
}

# Function to wait for and download training logs
wait_and_download_logs() {
    echo -e "${GREEN}Waiting for training to start...${NC}"
    
    # Initialize variables
    local max_attempts=60  # 5 minutes total (5 seconds * 60)
    local attempt=1
    local log_stream=""
    local spinner=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
    local spinner_idx=0
    
    # Function to show spinner
    show_spinner() {
        printf "\r${spinner[$spinner_idx]} Checking for training logs... (Attempt %d/%d)" $attempt $max_attempts
        spinner_idx=$(( (spinner_idx + 1) % ${#spinner[@]} ))
    }
    
    # Function to check for log stream
    check_log_stream() {
        log_stream=$(aws logs describe-log-streams \
            --log-group-name "$LOG_GROUP" \
            --order-by LastEventTime \
            --descending \
            --limit 1 \
            --query 'logStreams[0].logStreamName' \
            --output text \
            --region "$AWS_REGION")
        
        if [ "$log_stream" != "None" ]; then
            return 0
        else
            return 1
        fi
    }
    
    # Poll for logs with spinner
    while [ $attempt -le $max_attempts ]; do
        show_spinner
        
        if check_log_stream; then
            printf "\n${GREEN}Training logs found!${NC}\n"
            download_logs "$log_stream"
            return 0
        fi
        
        sleep 5
        attempt=$((attempt + 1))
    done
    
    printf "\n${YELLOW}No training logs found after 5 minutes.${NC}\n"
    echo -e "${YELLOW}You can check the AWS ECS console for more information:${NC}"
    echo -e "${GREEN}https://${AWS_REGION}.console.aws.amazon.com/ecs/home?region=${AWS_REGION}#/clusters/${CLUSTER_NAME}/services/${SERVICE_NAME}/details${NC}"
    
    # Ask user if they want to continue waiting
    read -p "Do you want to continue waiting? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        wait_and_download_logs  # Recursive call to start over
    else
        echo -e "${YELLOW}Stopped waiting for logs. You can check the AWS console for updates.${NC}"
    fi
}

# Main deployment process
echo -e "${GREEN}Starting deployment process...${NC}"

# Check Docker daemon first
check_docker_daemon

# Check AWS CLI
check_aws_cli

# Check ECR repository
check_ecr_repo

# Build Docker image with correct architecture
echo -e "${GREEN}Building Docker image...${NC}"
cd /Users/mukeshjoshi/gitprojects/Contract\ Management\ System\ /docker
docker buildx build --platform linux/amd64 -t ${REPOSITORY_NAME}:${IMAGE_TAG} . >/dev/null 2>&1

# Authenticate with ECR
echo -e "${GREEN}Authenticating with ECR...${NC}"
aws ecr get-login-password --region ${AWS_REGION} --no-cli-pager | docker login --username AWS --password-stdin ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com >/dev/null 2>&1

# Tag and push the image
echo -e "${GREEN}Tagging image...${NC}"
docker tag ${REPOSITORY_NAME}:${IMAGE_TAG} ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${REPOSITORY_NAME}:${IMAGE_TAG} >/dev/null 2>&1

echo -e "${GREEN}Pushing image to ECR...${NC}"
docker push ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${REPOSITORY_NAME}:${IMAGE_TAG} >/dev/null 2>&1

# Update ECS service
echo -e "${GREEN}Updating ECS service...${NC}"
aws ecs update-service --cluster contract-management-cluster --service mnist-training-service --force-new-deployment --region ${AWS_REGION} --no-cli-pager >/dev/null 2>&1

# Check deployment status
check_deployment_status

# Wait for and download training logs
wait_and_download_logs 