# AWS Nitro Enclaves Infrastructure

This directory contains Terraform configurations to deploy a secure AWS Nitro Enclaves infrastructure for training machine learning models in an isolated environment.

## Infrastructure Components

The infrastructure consists of the following components:

1. **VPC and Networking**
   - VPC with public subnet
   - Internet Gateway
   - Route Table
   - Security Group (SSH access)

2. **S3 Bucket**
   - Encrypted storage for training data
   - Versioning enabled
   - Server-side encryption
   - Public access blocked

3. **KMS Key**
   - Customer managed key for encryption
   - Key rotation enabled
   - IAM policies for EC2 access

4. **IAM Role and Policy**
   - EC2 instance role
   - S3 access permissions
   - KMS encryption/decryption permissions

5. **EC2 Instance**
   - Nitro Enclaves enabled
   - c5.xlarge instance type
   - 50GB GP3 root volume
   - User data script for Nitro Enclaves setup

6. **SSH Key Pair**
   - Automatically generated RSA key pair
   - Private key saved locally with secure permissions
   - Public key registered with AWS

## Directory Structure

```
enclave/
├── terraform/
│   ├── main.tf              # Main Terraform configuration
│   ├── variables.tf         # Input variables
│   ├── outputs.tf           # Output values
│   ├── scripts/
│   │   └── find_ami.sh     # Script to find latest Nitro Enclaves AMI
│   └── modules/
│       ├── vpc/            # VPC and networking
│       ├── s3/             # S3 bucket configuration
│       ├── kms/            # KMS key management
│       ├── iam/            # IAM roles and policies
│       └── ec2/            # EC2 instance configuration
```

## Prerequisites

1. AWS CLI configured with appropriate credentials
2. Terraform installed (version >= 1.0.0)

## Usage

1. **Find the Latest Nitro Enclaves AMI**
   ```bash
   cd terraform/scripts
   chmod +x find_ami.sh
   ./find_ami.sh
   ```
   This will output the latest AMI ID and instructions to add it to your terraform.tfvars file.

2. **Create terraform.tfvars**
   ```hcl
   aws_region = "us-east-2"
   bucket_name = "mmju_luks"  # Default bucket name
   key_name = "enclave-key"
   ami_id = "ami-xxxxxxxx"    # Add the AMI ID from the script output
   ```

3. **Initialize Terraform**
   ```bash
   cd terraform
   terraform init
   ```

4. **Plan the Deployment**
   ```bash
   terraform plan -var-file=terraform.tfvars
   ```

5. **Apply the Configuration**
   ```bash
   terraform apply -var-file=terraform.tfvars
   ```
   This will:
   - Generate a new SSH key pair
   - Save the private key as `enclave-key` (permissions: 600)
   - Save the public key as `enclave-key.pub` (permissions: 644)
   - Register the public key with AWS
   - Deploy all other infrastructure components

6. **Access the Instance**
   ```bash
   # Use the automatically generated private key to SSH into the instance
   ssh -i enclave-key ec2-user@<instance_public_ip>
   ```

## Variables

| Variable | Description | Default |
|----------|-------------|---------|
| aws_region | AWS region to deploy resources | us-east-2 |
| vpc_cidr | CIDR block for the VPC | 10.0.0.0/16 |
| bucket_name | Name of the S3 bucket | mmju_luks |
| kms_key_alias | Alias for the KMS key | enclave-training-key |
| key_name | Name of the SSH key pair | enclave-key |
| instance_type | EC2 instance type | c5.xlarge |
| root_volume_size | Size of the root volume in GB | 50 |
| root_volume_type | Type of the root volume | gp3 |
| ami_id | ID of the Nitro Enclaves AMI | (required) |

## Outputs

| Output | Description |
|--------|-------------|
| instance_public_ip | Public IP address of the EC2 instance |
| s3_bucket | Name of the S3 bucket |
| kms_key_id | ID of the KMS key (sensitive) |

## Security Features

1. **Network Security**
   - VPC with isolated subnet
   - Security group with minimal access (SSH only)
   - No public access to S3 bucket

2. **Data Security**
   - Server-side encryption for S3
   - KMS encryption for sensitive data
   - IAM roles with least privilege

3. **Instance Security**
   - Nitro Enclaves for secure processing
   - Encrypted root volume
   - Minimal IAM permissions

4. **SSH Key Security**
   - Automatically generated RSA key pair
   - Private key stored with secure permissions (600)
   - Public key stored with appropriate permissions (644)

## Cleanup

To destroy all resources:
```bash
terraform destroy -var-file=terraform.tfvars
```

## Notes

- The EC2 instance uses the latest Amazon Linux 2 AMI with Nitro Enclaves support
- The user data script automatically installs and configures the Nitro Enclaves CLI
- The S3 bucket name must be globally unique
- The KMS key has a 7-day deletion window
- All resources are tagged for easy identification
- SSH key pair is automatically generated and managed by Terraform
- Use the find_ami.sh script to get the latest Nitro Enclaves AMI ID

## Troubleshooting

1. **Instance Launch Issues**
   - Check the instance user data script
   - Verify security group rules
   - Ensure the AMI is available in your region
   - Verify the AMI ID is correct and from the us-east-2 region

2. **S3 Access Issues**
   - Verify IAM role permissions
   - Check bucket policy
   - Ensure bucket name is unique

3. **KMS Issues**
   - Verify key policy
   - Check IAM role permissions
   - Ensure key alias is unique

4. **SSH Key Issues**
   - Verify the private key file exists and has correct permissions (600)
   - Check that the public key was properly registered with AWS
   - Ensure you're using the correct private key file when connecting

## Contributing

Feel free to submit issues and enhancement requests! 