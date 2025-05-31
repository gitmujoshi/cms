#!/bin/bash
set -e

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Configuration
ENCLAVE_IMAGE_NAME="mnist-enclave"
ENCLAVE_IMAGE_TAG="latest"
ECR_REPOSITORY="contract-management-enclave"
AWS_REGION="us-west-2"
S3_BUCKET="contract-management-bucket-bkvww1ps"
KMS_KEY_ID="alias/mnist-training-key"

# Parse command line arguments
LOCAL_TEST=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --local)
            LOCAL_TEST=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Build the enclave image
echo "Building enclave image..."
cd "$PROJECT_ROOT/crates/enclave-runtime"
docker build -t $ENCLAVE_IMAGE_NAME:$ENCLAVE_IMAGE_TAG .

if [ "$LOCAL_TEST" = true ]; then
    echo "Running local test..."
    docker run --rm \
        -e S3_BUCKET=$S3_BUCKET \
        -e KMS_KEY_ID=$KMS_KEY_ID \
        -e MNIST_DATA_KEY="test_data" \
        $ENCLAVE_IMAGE_NAME:$ENCLAVE_IMAGE_TAG
    exit 0
fi

# Check if AWS CLI is configured
if ! aws sts get-caller-identity &>/dev/null; then
    echo "Error: AWS CLI is not configured properly"
    exit 1
fi

# Create ECR repository if it doesn't exist
if ! aws ecr describe-repositories --repository-names $ECR_REPOSITORY &>/dev/null; then
    echo "Creating ECR repository: $ECR_REPOSITORY"
    aws ecr create-repository --repository-name $ECR_REPOSITORY
fi

# Get ECR login token and login
aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $(aws sts get-caller-identity --query Account --output text).dkr.ecr.$AWS_REGION.amazonaws.com

# Tag and push the image to ECR
ECR_REPO_URI=$(aws sts get-caller-identity --query Account --output text).dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPOSITORY
docker tag $ENCLAVE_IMAGE_NAME:$ENCLAVE_IMAGE_TAG $ECR_REPO_URI:$ENCLAVE_IMAGE_TAG
docker push $ECR_REPO_URI:$ENCLAVE_IMAGE_TAG

# Create Nitro Enclave image
echo "Creating Nitro Enclave image..."
aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $(aws sts get-caller-identity --query Account --output text).dkr.ecr.$AWS_REGION.amazonaws.com
nitro-cli build-enclave --docker-uri $ECR_REPO_URI:$ENCLAVE_IMAGE_TAG --output-file enclave.eif

# Update ECS task definition
echo "Updating ECS task definition..."
TASK_DEF=$(cat <<EOF
{
    "family": "mnist-enclave-task",
    "networkMode": "awsvpc",
    "requiresCompatibilities": ["EC2"],
    "cpu": "2048",
    "memory": "4096",
    "containerDefinitions": [
        {
            "name": "enclave",
            "image": "$ECR_REPO_URI:$ENCLAVE_IMAGE_TAG",
            "essential": true,
            "environment": [
                {
                    "name": "S3_BUCKET",
                    "value": "$S3_BUCKET"
                },
                {
                    "name": "KMS_KEY_ID",
                    "value": "$KMS_KEY_ID"
                }
            ],
            "logConfiguration": {
                "logDriver": "awslogs",
                "options": {
                    "awslogs-group": "/ecs/mnist-enclave",
                    "awslogs-region": "$AWS_REGION",
                    "awslogs-stream-prefix": "ecs"
                }
            }
        }
    ]
}
EOF
)

# Register the task definition
TASK_DEF_ARN=$(aws ecs register-task-definition --cli-input-json "$TASK_DEF" --query 'taskDefinition.taskDefinitionArn' --output text)

# Update the service
echo "Updating ECS service..."
aws ecs update-service --cluster contract-management-cluster --service mnist-enclave-service --task-definition $TASK_DEF_ARN --force-new-deployment

echo "Deployment completed successfully!" 