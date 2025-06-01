#!/bin/bash
set -e

# Run training script
python3 mnist_train.py

# Upload model and metrics to S3
aws s3 cp mnist_*.h5 s3://${BUCKET_NAME}/models/
aws s3 cp mnist_metrics_*.json s3://${BUCKET_NAME}/metrics/
