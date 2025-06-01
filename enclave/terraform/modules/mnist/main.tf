resource "aws_s3_object" "mnist_model" {
  bucket = var.bucket_name
  key    = "models/mnist_model.py"
  content = <<-EOF
import tensorflow as tf
import numpy as np
import boto3
import os
import json
from datetime import datetime

# Initialize S3 client
s3 = boto3.client('s3')
BUCKET_NAME = os.environ.get('S3_BUCKET', '${var.bucket_name}')

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
}

resource "aws_s3_object" "requirements" {
  bucket = var.bucket_name
  key    = "models/requirements.txt"
  content = <<-EOF
tensorflow==2.12.0
numpy==1.23.5
boto3==1.26.137
EOF
}

resource "aws_s3_object" "dockerfile" {
  bucket = var.bucket_name
  key    = "models/Dockerfile"
  content = <<-EOF
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
COPY mnist_model.py .

# Set environment variables
ENV PYTHONUNBUFFERED=1

# Run the training script
CMD ["python", "mnist_model.py"]
EOF
}

resource "aws_s3_object" "deploy_script" {
  bucket = var.bucket_name
  key    = "models/deploy.sh"
  content = <<-EOF
#!/bin/bash

# Exit on error
set -e

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
} 