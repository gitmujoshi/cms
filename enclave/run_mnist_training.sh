#!/bin/bash

# Exit on error
set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

echo -e "${BLUE}Starting MNIST training automation...${NC}"

# Load variables from terraform output and tfvars
echo "Loading infrastructure details..."
INSTANCE_IP=$(cd terraform && terraform output -raw instance_public_ip)
BUCKET_NAME=$(grep s3_bucket terraform.tfvars | cut -d'=' -f2 | tr -d ' "')
KEY_PATH="$(pwd)/terraform/enclave-key"

# Create logs directory
mkdir -p logs

# Function to check if instance is ready
wait_for_instance() {
    echo -e "${BLUE}Waiting for instance to be ready...${NC}"
    while ! ssh -i "$KEY_PATH" -o StrictHostKeyChecking=no -o ConnectTimeout=5 ec2-user@$INSTANCE_IP echo "Instance is ready" &>/dev/null; do
        echo "Waiting for instance to be ready..."
        sleep 10
    done
}

# Function to run training
run_training() {
    echo -e "${BLUE}Starting MNIST training...${NC}"
    
    # Create remote script
    cat > remote_script.sh << 'EOF'
#!/bin/bash
set -e

# Create and enter model directory
mkdir -p ~/mnist_model
cd ~/mnist_model

# Download model files from S3
aws s3 cp s3://${BUCKET_NAME}/models/ . --recursive

# Make deploy script executable
chmod +x deploy.sh

# Run training and capture output
./deploy.sh 2>&1 | tee training_output.log

# Upload logs to S3
aws s3 cp training_output.log s3://${BUCKET_NAME}/logs/training_$(date +%Y%m%d_%H%M%S).log
EOF

    # Make script executable
    chmod +x remote_script.sh

    # Copy and run script on instance
    scp -i "$KEY_PATH" remote_script.sh ec2-user@$INSTANCE_IP:~
    ssh -i "$KEY_PATH" ec2-user@$INSTANCE_IP "BUCKET_NAME=$BUCKET_NAME ./remote_script.sh"
}

# Function to download logs
download_logs() {
    echo -e "${BLUE}Downloading training logs...${NC}"
    
    # Create logs directory if it doesn't exist
    mkdir -p logs
    
    # Download latest log file
    LATEST_LOG=$(aws s3 ls s3://$BUCKET_NAME/logs/ | sort | tail -n 1 | awk '{print $4}')
    aws s3 cp s3://$BUCKET_NAME/logs/$LATEST_LOG logs/
    
    echo -e "${GREEN}Logs downloaded to logs/$LATEST_LOG${NC}"
}

# Function to download model and metrics
download_results() {
    echo -e "${BLUE}Downloading model and metrics...${NC}"
    
    # Create results directory
    mkdir -p results
    
    # Download latest model and metrics
    LATEST_MODEL=$(aws s3 ls s3://$BUCKET_NAME/models/ | grep mnist_.*\.h5 | sort | tail -n 1 | awk '{print $4}')
    LATEST_METRICS=$(aws s3 ls s3://$BUCKET_NAME/metrics/ | grep mnist_.*\.json | sort | tail -n 1 | awk '{print $4}')
    
    aws s3 cp s3://$BUCKET_NAME/models/$LATEST_MODEL results/
    aws s3 cp s3://$BUCKET_NAME/metrics/$LATEST_METRICS results/
    
    echo -e "${GREEN}Model downloaded to results/$LATEST_MODEL${NC}"
    echo -e "${GREEN}Metrics downloaded to results/$LATEST_METRICS${NC}"
    
    # Display metrics
    echo -e "${BLUE}Training Metrics:${NC}"
    cat results/$LATEST_METRICS
}

# Main execution
echo -e "${BLUE}Starting MNIST training automation...${NC}"

# Wait for instance
wait_for_instance

# Run training
run_training

# Download logs and results
download_logs
download_results

echo -e "${GREEN}MNIST training completed successfully!${NC}"
echo -e "${BLUE}Results are available in:${NC}"
echo "- Logs: logs/"
echo "- Model and metrics: results/" 