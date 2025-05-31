#!/bin/bash

# Set variables
LOG_GROUP="/ecs/mnist-training"
LOG_STREAM="ecs/mnist-training/180c8428a8c34429a1e104328e8da0ec"
AWS_REGION="us-east-2"
OUTPUT_DIR="docs/training_logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Create output directory if it doesn't exist
mkdir -p "$OUTPUT_DIR"

# Download logs from CloudWatch
echo "Downloading logs from CloudWatch..."
aws logs get-log-events \
    --log-group-name "$LOG_GROUP" \
    --log-stream-name "$LOG_STREAM" \
    --region "$AWS_REGION" \
    --no-cli-pager > "$OUTPUT_DIR/training_logs_$TIMESTAMP.json"

# Convert JSON to readable format
echo "Converting logs to readable format..."
cat "$OUTPUT_DIR/training_logs_$TIMESTAMP.json" | jq -r '.events[] | .message' > "$OUTPUT_DIR/training_logs_$TIMESTAMP.txt"

# Copy the formatted logs to the main training_logs.txt
cp "$OUTPUT_DIR/training_logs_$TIMESTAMP.txt" training_logs.txt

echo "Logs have been downloaded and formatted successfully!"
echo "JSON logs saved to: $OUTPUT_DIR/training_logs_$TIMESTAMP.json"
echo "Formatted logs saved to: $OUTPUT_DIR/training_logs_$TIMESTAMP.txt"
echo "Main training logs updated at: training_logs.txt" 