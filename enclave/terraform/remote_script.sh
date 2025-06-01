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
