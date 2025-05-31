#!/bin/bash

# Exit on error
set -e

# Set AWS region and account
AWS_REGION="us-east-2"
AWS_ACCOUNT_ID="124355660528"
REPOSITORY_NAME="mnist-training"
CLUSTER_NAME="contract-management-cluster"
SERVICE_NAME="mnist-training-service"
LOG_GROUP="/ecs/mnist-training"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to check if AWS CLI is working
check_aws_cli() {
    echo -e "${GREEN}Checking AWS CLI...${NC}"
    if ! aws sts get-caller-identity >/dev/null 2>&1; then
        echo -e "${RED}Error: AWS CLI is not configured properly${NC}"
        exit 1
    fi
}

# Function to delete ECS service
delete_ecs_service() {
    echo -e "${GREEN}Deleting ECS service...${NC}"
    if aws ecs describe-services --cluster ${CLUSTER_NAME} --services ${SERVICE_NAME} --region ${AWS_REGION} >/dev/null 2>&1; then
        # Update service to 0 tasks
        aws ecs update-service --cluster ${CLUSTER_NAME} --service ${SERVICE_NAME} --desired-count 0 --region ${AWS_REGION} >/dev/null 2>&1
        echo -e "${GREEN}Waiting for tasks to drain...${NC}"
        sleep 30
        # Delete the service
        aws ecs delete-service --cluster ${CLUSTER_NAME} --service ${SERVICE_NAME} --force --region ${AWS_REGION} >/dev/null 2>&1
        echo -e "${GREEN}ECS service deleted successfully${NC}"
    else
        echo -e "${YELLOW}ECS service not found${NC}"
    fi
}

# Function to delete ECS cluster
delete_ecs_cluster() {
    echo -e "${GREEN}Deleting ECS cluster...${NC}"
    if aws ecs describe-clusters --clusters ${CLUSTER_NAME} --region ${AWS_REGION} >/dev/null 2>&1; then
        aws ecs delete-cluster --cluster ${CLUSTER_NAME} --region ${AWS_REGION} >/dev/null 2>&1
        echo -e "${GREEN}ECS cluster deleted successfully${NC}"
    else
        echo -e "${YELLOW}ECS cluster not found${NC}"
    fi
}

# Function to delete ECR repository
delete_ecr_repository() {
    echo -e "${GREEN}Deleting ECR repository...${NC}"
    if aws ecr describe-repositories --repository-names ${REPOSITORY_NAME} --region ${AWS_REGION} >/dev/null 2>&1; then
        # Delete all images in the repository
        aws ecr batch-delete-image \
            --repository-name ${REPOSITORY_NAME} \
            --image-ids "$(aws ecr list-images --repository-name ${REPOSITORY_NAME} --region ${AWS_REGION} --query 'imageIds[*]' --output json)" \
            --region ${AWS_REGION} >/dev/null 2>&1 || true
        
        # Delete the repository
        aws ecr delete-repository --repository-name ${REPOSITORY_NAME} --force --region ${AWS_REGION} >/dev/null 2>&1
        echo -e "${GREEN}ECR repository deleted successfully${NC}"
    else
        echo -e "${YELLOW}ECR repository not found${NC}"
    fi
}

# Function to delete CloudWatch log group
delete_log_group() {
    echo -e "${GREEN}Deleting CloudWatch log group...${NC}"
    if aws logs describe-log-groups --log-group-name-prefix ${LOG_GROUP} --region ${AWS_REGION} >/dev/null 2>&1; then
        aws logs delete-log-group --log-group-name ${LOG_GROUP} --region ${AWS_REGION} >/dev/null 2>&1
        echo -e "${GREEN}CloudWatch log group deleted successfully${NC}"
    else
        echo -e "${YELLOW}CloudWatch log group not found${NC}"
    fi
}

# Function to delete local files
cleanup_local_files() {
    echo -e "${GREEN}Cleaning up local files...${NC}"
    
    # Remove training logs
    if [ -f "../training_logs.txt" ]; then
        rm "../training_logs.txt"
        echo -e "${GREEN}Removed training logs${NC}"
    fi
    
    # Remove training reports
    if [ -d "../docs/training_logs" ]; then
        rm -rf "../docs/training_logs"
        echo -e "${GREEN}Removed training reports${NC}"
    fi
}

# Main cleanup process
echo -e "${GREEN}Starting cleanup process...${NC}"

# Check AWS CLI
check_aws_cli

# Delete resources in reverse order of creation
delete_ecs_service
delete_ecs_cluster
delete_ecr_repository
delete_log_group
cleanup_local_files

echo -e "${GREEN}Cleanup completed successfully!${NC}"
echo -e "${GREEN}All resources have been deleted.${NC}" 