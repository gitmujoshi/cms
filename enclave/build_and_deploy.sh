#!/bin/bash

# Exit on error
set -e

# Configuration
ENCLAVE_INSTANCE_IP=""
ENCLAVE_KEY_PATH="./enclave-key"
S3_BUCKET="mmju-luks-2024"
MODEL_DIR="mnist_model"
ENCLAVE_IMAGE_NAME="mnist-training-enclave"
ENCLAVE_PORT=5000

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status messages
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if instance IP is provided
if [ -z "$ENCLAVE_INSTANCE_IP" ]; then
    print_error "Please set ENCLAVE_INSTANCE_IP in the script"
    exit 1
fi

# Check if key file exists
if [ ! -f "$ENCLAVE_KEY_PATH" ]; then
    print_error "SSH key not found at $ENCLAVE_KEY_PATH"
    exit 1
fi

# Create model directory if it doesn't exist
mkdir -p $MODEL_DIR

# Create Dockerfile for the enclave
print_status "Creating Dockerfile for MNIST training..."
cat > Dockerfile << 'EOF'
FROM python:3.9-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy training script
COPY mnist_train.py .

# Set environment variables
ENV PYTHONUNBUFFERED=1

# Run the training script
CMD ["python", "mnist_train.py"]
EOF

# Create requirements.txt
print_status "Creating requirements.txt..."
cat > requirements.txt << 'EOF'
tensorflow==2.12.0
numpy==1.23.5
boto3==1.26.137
EOF

# Create MNIST training script
print_status "Creating MNIST training script..."
cat > mnist_train.py << 'EOF'
import tensorflow as tf
import numpy as np
import boto3
import os
import json
from datetime import datetime

# Initialize S3 client
s3 = boto3.client('s3')
BUCKET_NAME = os.environ.get('S3_BUCKET', 'mmju-luks-2024')

def load_mnist_data():
    print("Loading MNIST dataset...")
    (x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()
    x_train = x_train.astype('float32') / 255.0
    x_test = x_test.astype('float32') / 255.0
    return (x_train, y_train), (x_test, y_test)

def create_model():
    print("Creating model...")
    model = tf.keras.Sequential([
        tf.keras.layers.Flatten(input_shape=(28, 28)),
        tf.keras.layers.Dense(128, activation='relu'),
        tf.keras.layers.Dropout(0.2),
        tf.keras.layers.Dense(10, activation='softmax')
    ])
    
    model.compile(optimizer='adam',
                 loss='sparse_categorical_crossentropy',
                 metrics=['accuracy'])
    return model

def train_model(model, x_train, y_train, x_test, y_test):
    print("Training model...")
    history = model.fit(x_train, y_train,
                       epochs=5,
                       validation_data=(x_test, y_test),
                       verbose=1)
    return history

def save_model(model, history):
    print("Saving model and metrics...")
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    
    # Save model locally
    model.save('model.h5')
    
    # Save metrics
    metrics = {
        'accuracy': float(history.history['accuracy'][-1]),
        'val_accuracy': float(history.history['val_accuracy'][-1]),
        'loss': float(history.history['loss'][-1]),
        'val_loss': float(history.history['val_loss'][-1]),
        'timestamp': timestamp
    }
    
    with open('metrics.json', 'w') as f:
        json.dump(metrics, f)
    
    # Upload to S3
    s3.upload_file('model.h5', BUCKET_NAME, f'models/mnist_{timestamp}.h5')
    s3.upload_file('metrics.json', BUCKET_NAME, f'metrics/mnist_{timestamp}.json')
    
    print(f"Model and metrics saved to S3 bucket: {BUCKET_NAME}")

def main():
    print("Starting MNIST training in Nitro Enclave...")
    
    # Load data
    (x_train, y_train), (x_test, y_test) = load_mnist_data()
    
    # Create and train model
    model = create_model()
    history = train_model(model, x_train, y_train, x_test, y_test)
    
    # Save results
    save_model(model, history)
    
    print("Training completed successfully!")

if __name__ == "__main__":
    main()
EOF

# Build the enclave image
print_status "Building enclave image..."
docker build -t $ENCLAVE_IMAGE_NAME .

# Copy files to EC2 instance
print_status "Copying files to EC2 instance..."
scp -i $ENCLAVE_KEY_PATH -r $MODEL_DIR ec2-user@$ENCLAVE_INSTANCE_IP:~/
scp -i $ENCLAVE_KEY_PATH Dockerfile requirements.txt mnist_train.py ec2-user@$ENCLAVE_INSTANCE_IP:~/

# SSH into the instance and run the enclave
print_status "SSH into instance and starting enclave..."
ssh -i $ENCLAVE_KEY_PATH ec2-user@$ENCLAVE_INSTANCE_IP << 'EOF'
    # Install Docker if not installed
    if ! command -v docker &> /dev/null; then
        sudo yum update -y
        sudo yum install -y docker
        sudo systemctl start docker
        sudo systemctl enable docker
        sudo usermod -aG docker ec2-user
    fi

    # Build the enclave image
    docker build -t mnist-training-enclave .

    # Create and run the enclave
    nitro-cli build-enclave --docker-uri mnist-training-enclave:latest --output-file mnist-training.eif

    # Run the enclave
    nitro-cli run-enclave --eif-path mnist-training.eif --memory 2048 --cpu-count 2 --debug-mode

    # Get the enclave ID
    ENCLAVE_ID=$(nitro-cli describe-enclaves | jq -r '.[0].EnclaveID')

    # Attach console to see output
    nitro-cli console --enclave-id $ENCLAVE_ID
EOF

print_status "Deployment completed!"
print_status "You can monitor the training progress by checking the S3 bucket: $S3_BUCKET"
print_status "The model and metrics will be saved in the 'models' and 'metrics' folders respectively" 