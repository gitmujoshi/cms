#!/bin/bash

# Set the region
REGION="us-east-2"

echo "Searching for latest Amazon Linux 2 AMI with Nitro Enclaves support in $REGION..."

# Get the latest Amazon Linux 2 AMI
AMI_INFO=$(aws ec2 describe-images \
    --region $REGION \
    --owners amazon \
    --filters "Name=name,Values=amzn2-ami-hvm-*-x86_64-gp2" "Name=state,Values=available" \
    --query 'sort_by(Images, &CreationDate)[-1].[ImageId,Name,CreationDate]' \
    --output text 2>/dev/null)

if [ $? -ne 0 ]; then
    echo "Error: Failed to query AWS for AMIs. Please check your AWS credentials and permissions."
    exit 1
fi

if [ -z "$AMI_INFO" ]; then
    echo "Error: No Amazon Linux 2 AMI found in $REGION"
    exit 1
fi

# Split the output into variables
read -r AMI_ID AMI_NAME CREATION_DATE <<< "$AMI_INFO"

if [ "$AMI_ID" = "None" ] || [ -z "$AMI_ID" ]; then
    echo "Error: No valid Amazon Linux 2 AMI found in $REGION"
    exit 1
fi

echo "Found latest Amazon Linux 2 AMI:"
echo "AMI ID: $AMI_ID"
echo "Name: $AMI_NAME"
echo "Creation Date: $CREATION_DATE"

echo -e "\nNote: This AMI supports Nitro Enclaves. The Nitro Enclaves software will be installed via user data script."

# Output in a format suitable for terraform.tfvars
echo -e "\nAdd this to your terraform.tfvars file:"
echo "ami_id = \"$AMI_ID\"" 