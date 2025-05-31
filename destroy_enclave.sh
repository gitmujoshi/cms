#!/bin/bash

# Exit on error
set -e

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

# Check if enclave_info.txt exists
if [ ! -f enclave_info.txt ]; then
    print_error "enclave_info.txt not found. Please run setup_enclave.sh first."
    exit 1
fi

# Source the enclave information
source enclave_info.txt

# Function to check if a resource exists
resource_exists() {
    local resource_type=$1
    local resource_id=$2
    local query=$3
    
    if aws ec2 describe-$resource_type --$resource_type-ids $resource_id --query "$query" --output text 2>/dev/null; then
        return 0
    else
        return 1
    fi
}

# Terminate EC2 instance if it exists
if resource_exists "instances" "$INSTANCE_ID" "Reservations[0].Instances[0].State.Name"; then
    print_status "Terminating EC2 instance..."
    aws ec2 terminate-instances --instance-ids $INSTANCE_ID
    aws ec2 wait instance-terminated --instance-ids $INSTANCE_ID
else
    print_warning "EC2 instance $INSTANCE_ID not found or already terminated"
fi

# Delete security group if it exists
if resource_exists "security-groups" "$SECURITY_GROUP_ID" "SecurityGroups[0].GroupId"; then
    print_status "Deleting security group..."
    aws ec2 delete-security-group --group-id $SECURITY_GROUP_ID
else
    print_warning "Security group $SECURITY_GROUP_ID not found or already deleted"
fi

# Delete IAM role and instance profile
print_status "Cleaning up IAM resources..."
if aws iam get-role --role-name ${ENCLAVE_NAME}-role 2>/dev/null; then
    # Detach policies
    aws iam detach-role-policy --role-name ${ENCLAVE_NAME}-role \
        --policy-arn arn:aws:iam::aws:policy/AmazonECR-ReadOnly

    # Remove role from instance profile
    aws iam remove-role-from-instance-profile \
        --instance-profile-name ${ENCLAVE_NAME}-profile \
        --role-name ${ENCLAVE_NAME}-role

    # Delete instance profile
    aws iam delete-instance-profile --instance-profile-name ${ENCLAVE_NAME}-profile

    # Delete role
    aws iam delete-role --role-name ${ENCLAVE_NAME}-role
else
    print_warning "IAM role ${ENCLAVE_NAME}-role not found or already deleted"
fi

# Delete ECR repository
print_status "Deleting ECR repository..."
if aws ecr describe-repositories --repository-names ${ENCLAVE_NAME} 2>/dev/null; then
    aws ecr delete-repository --repository-name ${ENCLAVE_NAME} --force
else
    print_warning "ECR repository ${ENCLAVE_NAME} not found or already deleted"
fi

# Remove enclave_info.txt
rm -f enclave_info.txt

print_status "Cleanup completed successfully!" 