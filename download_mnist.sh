#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create data directory
DATA_DIR="mnist_data"
echo -e "${GREEN}Creating data directory...${NC}"
mkdir -p "$DATA_DIR"

# Download MNIST data
echo -e "${GREEN}Downloading MNIST dataset...${NC}"
curl -L -o "$DATA_DIR/mnist.npz" https://storage.googleapis.com/tensorflow/tf-keras-datasets/mnist.npz

# Check if download was successful
if [ ! -f "$DATA_DIR/mnist.npz" ]; then
    echo -e "${YELLOW}Error: Failed to download MNIST dataset${NC}"
    exit 1
fi

# Create a Python script to process the data
echo -e "${GREEN}Creating processing script...${NC}"
cat > "$DATA_DIR/process_data.py" << 'EOF'
import numpy as np
import matplotlib.pyplot as plt
from datetime import datetime
import os

# Load the data
print("Loading MNIST data...")
with np.load('mnist.npz') as data:
    x_train = data['x_train']
    y_train = data['y_train']
    x_test = data['x_test']
    y_test = data['y_test']

# Save individual files
print("Saving data to files...")
np.save('x_train.npy', x_train)
np.save('y_train.npy', y_train)
np.save('x_test.npy', x_test)
np.save('y_test.npy', y_test)

# Create summary file
print("Creating summary...")
with open('dataset_summary.txt', 'w') as f:
    f.write(f"MNIST Dataset Summary\n")
    f.write(f"Generated on: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
    f.write(f"Training Data:\n")
    f.write(f"- Number of samples: {len(x_train)}\n")
    f.write(f"- Image shape: {x_train[0].shape}\n")
    f.write(f"- Value range: [{x_train.min()}, {x_train.max()}]\n\n")
    f.write(f"Test Data:\n")
    f.write(f"- Number of samples: {len(x_test)}\n")
    f.write(f"- Image shape: {x_test[0].shape}\n")
    f.write(f"- Value range: [{x_test.min()}, {x_test.max()}]\n\n")
    f.write(f"Class Distribution (Training):\n")
    for i in range(10):
        count = np.sum(y_train == i)
        f.write(f"- Digit {i}: {count} samples ({count/len(y_train)*100:.2f}%)\n")

# Create visualization
print("Creating visualizations...")
plt.figure(figsize=(15, 5))
for i in range(10):
    plt.subplot(2, 5, i+1)
    plt.imshow(x_train[i], cmap='gray')
    plt.title(f'Digit: {y_train[i]}')
    plt.axis('off')
plt.tight_layout()
plt.savefig('sample_images.png')
print("Done!")
EOF

# Create requirements file
echo -e "${GREEN}Creating requirements file...${NC}"
cat > "$DATA_DIR/requirements.txt" << 'EOF'
numpy>=1.21.0
matplotlib>=3.5.0
EOF

# Install requirements and run processing script
echo -e "${GREEN}Installing requirements...${NC}"
cd "$DATA_DIR"
pip install -r requirements.txt

echo -e "${GREEN}Processing data...${NC}"
python process_data.py

# Clean up
echo -e "${GREEN}Cleaning up...${NC}"
rm process_data.py requirements.txt

echo -e "${GREEN}Data download and processing complete!${NC}"
echo -e "Files created in ${DATA_DIR}/:"
echo "- x_train.npy: Training images"
echo "- y_train.npy: Training labels"
echo "- x_test.npy: Test images"
echo "- y_test.npy: Test labels"
echo "- dataset_summary.txt: Dataset statistics"
echo "- sample_images.png: Visualization of sample images" 