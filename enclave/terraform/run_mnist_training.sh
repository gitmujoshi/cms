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
INSTANCE_IP=$(terraform output -raw instance_public_ip)
BUCKET_NAME=$(grep s3_bucket terraform.tfvars | cut -d'=' -f2 | tr -d ' "')
KEY_PATH="$(pwd)/enclave-key"

# Create logs directory
mkdir -p logs

# Function to view instance console output
view_console_output() {
    echo -e "${BLUE}Fetching instance console output...${NC}"
    aws ec2 get-console-output --instance-id $(terraform output -raw instance_id) | jq -r '.Output' | grep -E 'Installing|Starting|Failed|Error|Complete|Cloud-init|Docker|Nitro|Enclave|aws-nitro-enclaves-cli' | tail -n 20
}

# Function to check if instance is ready
wait_for_instance() {
    echo -e "${BLUE}Waiting for instance to be ready...${NC}"
    
    # First wait for SSH access
    while ! ssh -i "$KEY_PATH" -o StrictHostKeyChecking=no -o ConnectTimeout=5 ec2-user@$INSTANCE_IP echo "SSH is ready" &>/dev/null; do
        echo "Waiting for SSH access..."
        view_console_output
        sleep 10
    done
    
    # Then wait for Docker to be available
    echo "Waiting for Docker to be ready..."
    while ! ssh -i "$KEY_PATH" ec2-user@$INSTANCE_IP "docker --version" &>/dev/null; do
        echo "Waiting for Docker installation..."
        view_console_output
        sleep 10
    done
    
    # Finally wait for Nitro CLI to be available
    echo "Waiting for Nitro CLI to be ready..."
    while ! ssh -i "$KEY_PATH" ec2-user@$INSTANCE_IP "nitro-cli --version" &>/dev/null; do
        echo "Waiting for Nitro CLI installation..."
        view_console_output
        sleep 10
    done
    
    echo -e "${GREEN}Instance is ready with Docker and Nitro CLI installed${NC}"
}

# Function to copy required files to instance
copy_files_to_instance() {
    echo -e "${BLUE}Copying required files to instance...${NC}"
    
    # Create a temporary directory for files
    TEMP_DIR=$(mktemp -d)
    cd "$TEMP_DIR"
    
    # Create mnist_model directory
    mkdir -p mnist_model
    cd mnist_model
    
    # Create Dockerfile
    cat > Dockerfile << 'EOF'
FROM amazonlinux:2

# Install required packages
RUN yum update -y && \
    yum install -y python3-pip python3-devel gcc && \
    yum clean all

# Install Python packages
COPY requirements.txt .
RUN pip3 install -r requirements.txt

# Copy training script
COPY mnist_train.py .

# Set working directory
WORKDIR /app

# Add Nitro Enclaves specific configuration
ENV NITRO_ENCLAVE=1
ENV AWS_NITRO_ENCLAVES_ENABLED=1

# Run training script
CMD ["python3", "mnist_train.py"]
EOF

    # Create requirements.txt with compatible versions
    cat > requirements.txt << 'EOF'
tensorflow==2.11.0
numpy==1.21.6
pandas==1.3.5
EOF

    # Create training script
    cat > mnist_train.py << 'EOF'
import tensorflow as tf
import numpy as np
import os
import json
from datetime import datetime

# Load MNIST dataset
(x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()

# Normalize pixel values
x_train = x_train.astype('float32') / 255
x_test = x_test.astype('float32') / 255

# Create model
model = tf.keras.Sequential([
    tf.keras.layers.Flatten(input_shape=(28, 28)),
    tf.keras.layers.Dense(128, activation='relu'),
    tf.keras.layers.Dropout(0.2),
    tf.keras.layers.Dense(10, activation='softmax')
])

# Compile model
model.compile(optimizer='adam',
              loss='sparse_categorical_crossentropy',
              metrics=['accuracy'])

# Train model
history = model.fit(x_train, y_train, epochs=5, validation_split=0.2)

# Evaluate model
test_loss, test_acc = model.evaluate(x_test, y_test)
print(f'\nTest accuracy: {test_acc}')

# Save model
timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
model.save(f'mnist_{timestamp}.h5')

# Save metrics
metrics = {
    'test_accuracy': float(test_acc),
    'test_loss': float(test_loss),
    'training_history': history.history,
    'timestamp': timestamp
}

with open(f'mnist_metrics_{timestamp}.json', 'w') as f:
    json.dump(metrics, f, indent=2)
EOF

    # Create deploy script
    cat > deploy.sh << 'EOF'
#!/bin/bash
set -e

# Run training script
python3 mnist_train.py

# Upload model and metrics to S3
aws s3 cp mnist_*.h5 s3://${BUCKET_NAME}/models/
aws s3 cp mnist_metrics_*.json s3://${BUCKET_NAME}/metrics/
EOF

    # Make deploy script executable
    chmod +x deploy.sh

    # Copy all files to instance
    cd "$TEMP_DIR"
    scp -i "$KEY_PATH" -r mnist_model ec2-user@$INSTANCE_IP:~/
    
    # Clean up
    cd "$SCRIPT_DIR"
    rm -rf "$TEMP_DIR"
}

# Function to run training in enclave
run_training() {
    echo -e "${BLUE}Starting MNIST training in Nitro Enclave...${NC}"
    
    # Create remote script
    cat > remote_script.sh << 'EOF'
#!/bin/bash
set -e

cd ~/mnist_model

# Build the enclave image
echo "Building enclave image..."
docker build -t mnist-enclave .

# Create the enclave image
echo "Creating enclave image..."
nitro-cli build-enclave --docker-uri mnist-enclave:latest --output-file mnist-enclave.eif

# Allocate memory for the enclave (4GB)
echo "Allocating memory for enclave..."
nitro-cli allocate-memory --memory 4096

# Run the enclave
echo "Starting enclave..."
ENCLAVE_ID=$(nitro-cli run-enclave --eif-path mnist-enclave.eif --cpu-count 2 --memory 4096 --debug-mode | jq -r '.EnclaveID')

# Wait for enclave to be ready
echo "Waiting for enclave to be ready..."
sleep 10

# Run training in the enclave
echo "Running MNIST training in enclave..."
nitro-cli console --enclave-id $ENCLAVE_ID

# Capture output
./deploy.sh 2>&1 | tee training_output.log

# Upload logs to S3
aws s3 cp training_output.log s3://${BUCKET_NAME}/logs/training_$(date +%Y%m%d_%H%M%S).log

# Terminate enclave
echo "Terminating enclave..."
nitro-cli terminate-enclave --enclave-id $ENCLAVE_ID
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

# Copy required files
copy_files_to_instance

# Run training
run_training

# Download logs and results
download_logs
download_results

echo -e "${GREEN}MNIST training completed successfully!${NC}"
echo -e "${BLUE}Results are available in:${NC}"
echo "- Logs: logs/"
echo "- Model and metrics: results/" 