#!/bin/bash

# Exit on error
set -e

# Configuration
AWS_REGION="us-east-2"
ENCLAVE_NAME="mnist-training-enclave"
PARENT_INSTANCE_TYPE="c5.xlarge"  # Minimum instance type that supports Nitro Enclaves
ENCLAVE_CPU_COUNT=2
ENCLAVE_MEMORY_MIB=4096
ENCLAVE_IMAGE_NAME="mnist-training-enclave"
ENCLAVE_IMAGE_TAG="latest"
SETUP_INSTANCE_TYPE="t2.micro"  # Small instance for setup
KEY_NAME="${ENCLAVE_NAME}-key"

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

# Check if AWS CLI is installed
if ! command -v aws &> /dev/null; then
    print_error "AWS CLI is not installed. Please install it first."
    exit 1
fi

# Create key pair if it doesn't exist
print_status "Creating key pair if it doesn't exist..."
if ! aws ec2 describe-key-pairs --region ${AWS_REGION} --key-names ${KEY_NAME} > /dev/null 2>&1; then
    aws ec2 create-key-pair \
        --region ${AWS_REGION} \
        --key-name ${KEY_NAME} \
        --query 'KeyMaterial' \
        --output text > ${KEY_NAME}.pem
    chmod 400 ${KEY_NAME}.pem
    print_status "Key pair created and saved to ${KEY_NAME}.pem"
else
    print_status "Key pair already exists."
fi

# Get AWS account ID
AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)

# Create ECR repository if it doesn't exist
print_status "Creating ECR repository if it doesn't exist..."
if ! aws ecr describe-repositories --region ${AWS_REGION} --repository-names ${ENCLAVE_IMAGE_NAME} > /dev/null 2>&1; then
    aws ecr create-repository --region ${AWS_REGION} --repository-name ${ENCLAVE_IMAGE_NAME}
    print_status "ECR repository created."
else
    print_status "ECR repository already exists."
fi

# Wait for ECR repository to be available
until aws ecr describe-repositories --region ${AWS_REGION} --repository-names ${ENCLAVE_IMAGE_NAME} > /dev/null 2>&1; do
    print_status "Waiting for ECR repository to be available..."
    sleep 2
done

# Get ECR login token and login to Docker
print_status "Logging in to ECR..."
aws ecr get-login-password --region ${AWS_REGION} | \
    docker login --username AWS --password-stdin ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com

# Build the enclave image
print_status "Building enclave image..."
docker build -t ${ENCLAVE_IMAGE_NAME}:${ENCLAVE_IMAGE_TAG} -f Dockerfile.enclave .

# Tag the image for ECR
docker tag ${ENCLAVE_IMAGE_NAME}:${ENCLAVE_IMAGE_TAG} ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ENCLAVE_IMAGE_NAME}:${ENCLAVE_IMAGE_TAG}

# Push the image to ECR
print_status "Pushing image to ECR..."
docker push ${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com/${ENCLAVE_IMAGE_NAME}:${ENCLAVE_IMAGE_TAG}

# Create IAM role for the enclave
print_status "Creating IAM role for the enclave..."
if ! aws iam get-role --role-name ${ENCLAVE_NAME}-role > /dev/null 2>&1; then
    aws iam create-role --role-name ${ENCLAVE_NAME}-role \
        --assume-role-policy-document '{
            "Version": "2012-10-17",
            "Statement": [{
                "Effect": "Allow",
                "Principal": {
                    "Service": "ec2.amazonaws.com"
                },
                "Action": "sts:AssumeRole"
            }]
        }'
    print_status "IAM role created."
else
    print_status "IAM role already exists."
fi

# Attach necessary policies
print_status "Attaching policies to IAM role..."
aws iam attach-role-policy --role-name ${ENCLAVE_NAME}-role \
    --policy-arn arn:aws:iam::aws:policy/AmazonECR-FullAccess || true

# Create instance profile
print_status "Creating instance profile..."
if ! aws iam get-instance-profile --instance-profile-name ${ENCLAVE_NAME}-profile > /dev/null 2>&1; then
    aws iam create-instance-profile --instance-profile-name ${ENCLAVE_NAME}-profile
    aws iam add-role-to-instance-profile \
        --instance-profile-name ${ENCLAVE_NAME}-profile \
        --role-name ${ENCLAVE_NAME}-role
    print_status "Instance profile created and role added."
else
    print_status "Instance profile already exists."
fi

# Wait for instance profile to be available
until aws iam get-instance-profile --instance-profile-name ${ENCLAVE_NAME}-profile > /dev/null 2>&1; do
    print_status "Waiting for instance profile to be available..."
    sleep 2
done

# Create security group
print_status "Creating security group..."
VPC_ID=$(aws ec2 describe-vpcs --region ${AWS_REGION} --query 'Vpcs[0].VpcId' --output text)
if ! aws ec2 describe-security-groups --region ${AWS_REGION} --group-names ${ENCLAVE_NAME}-sg > /dev/null 2>&1; then
    SECURITY_GROUP_ID=$(aws ec2 create-security-group \
        --region ${AWS_REGION} \
        --group-name ${ENCLAVE_NAME}-sg \
        --description "Security group for ${ENCLAVE_NAME}" \
        --vpc-id ${VPC_ID} \
        --query 'GroupId' \
        --output text)
    print_status "Security group created."
else
    SECURITY_GROUP_ID=$(aws ec2 describe-security-groups --region ${AWS_REGION} --group-names ${ENCLAVE_NAME}-sg --query 'SecurityGroups[0].GroupId' --output text)
    print_status "Security group already exists."
fi

# Allow necessary inbound traffic
aws ec2 authorize-security-group-ingress \
    --region ${AWS_REGION} \
    --group-id ${SECURITY_GROUP_ID} \
    --protocol tcp \
    --port 22 \
    --cidr 0.0.0.0/0 || true

# Create EC2 instance
print_status "Creating EC2 instance..."
INSTANCE_ID=$(aws ec2 run-instances \
    --region ${AWS_REGION} \
    --image-id ami-0c55b159cbfafe1f0 \
    --instance-type ${PARENT_INSTANCE_TYPE} \
    --security-group-ids ${SECURITY_GROUP_ID} \
    --iam-instance-profile Name=${ENCLAVE_NAME}-profile \
    --key-name ${KEY_NAME} \
    --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=${ENCLAVE_NAME}}]" \
    --query 'Instances[0].InstanceId' \
    --output text)

# Wait for instance to be running
print_status "Waiting for instance to be running..."
aws ec2 wait instance-running --region ${AWS_REGION} --instance-ids ${INSTANCE_ID}

# Get instance public IP
INSTANCE_IP=$(aws ec2 describe-instances \
    --region ${AWS_REGION} \
    --instance-ids ${INSTANCE_ID} \
    --query 'Reservations[0].Instances[0].PublicIpAddress' \
    --output text)

print_status "Enclave setup completed successfully!"
print_status "Instance ID: ${INSTANCE_ID}"
print_status "Instance IP: ${INSTANCE_IP}"
print_status "Security Group ID: ${SECURITY_GROUP_ID}"

# Save the instance information to a file for cleanup
cat > enclave_info.txt << EOF
INSTANCE_ID=${INSTANCE_ID}
SECURITY_GROUP_ID=${SECURITY_GROUP_ID}
ENCLAVE_NAME=${ENCLAVE_NAME}
AWS_REGION=${AWS_REGION}
KEY_NAME=${KEY_NAME}
EOF

print_status "Setup information saved to enclave_info.txt"
print_status "You can now SSH into the instance and start the enclave"
print_status "SSH command: ssh -i ${KEY_NAME}.pem ec2-user@${INSTANCE_IP}"

# Create a setup script to be run on the instance
cat > setup_instance.sh << 'EOF'
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
EOF

chmod +x setup_instance.sh

print_status "Created setup_instance.sh script. Please copy it to the instance and run it:"
print_status "scp -i ${KEY_NAME}.pem setup_instance.sh ec2-user@${INSTANCE_IP}:~/"
print_status "ssh -i ${KEY_NAME}.pem ec2-user@${INSTANCE_IP}"
print_status "Then run: ./setup_instance.sh" 