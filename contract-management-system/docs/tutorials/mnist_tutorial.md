# MNIST Tutorial: Secure Model Training

This tutorial demonstrates how to train a convolutional neural network (CNN) on the MNIST dataset using our secure training system.

## Overview

We'll cover:
1. Setting up the environment
2. Preparing the MNIST dataset
3. Defining a CNN model
4. Training securely in an enclave
5. Evaluating the model
6. Deploying the trained model

## Prerequisites

- Installed DCMS system
- AWS account with Nitro Enclaves access
- Python 3.8+ (for data preparation)
- 8GB+ RAM
- CUDA-capable GPU (optional)

## 1. Environment Setup

```bash
# Clone the repository if you haven't already
git clone https://github.com/your-org/contract-management-system.git
cd contract-management-system

# Build the training binary
cd crates/model-training
cargo build --release

# Set up environment variables
export RUST_LOG=info
export AWS_REGION=us-west-2
export ENCLAVE_CPU_COUNT=4
export ENCLAVE_MEMORY_MIB=8192
```

## 2. Data Preparation

Create a Python script `prepare_mnist.py`:

```python
import torch
from torchvision import datasets, transforms
import numpy as np
import pyarrow as pa
import pyarrow.parquet as pq
from pathlib import Path

def prepare_mnist_data():
    # Create data directories
    data_dir = Path("data/mnist")
    data_dir.mkdir(parents=True, exist_ok=True)
    
    # Define transforms
    transform = transforms.Compose([
        transforms.ToTensor(),
        transforms.Normalize((0.1307,), (0.3081,))
    ])
    
    # Download and load MNIST
    train_dataset = datasets.MNIST('data/raw', train=True, download=True, transform=transform)
    test_dataset = datasets.MNIST('data/raw', train=False, transform=transform)
    
    # Convert to numpy arrays
    train_images = train_dataset.data.numpy()
    train_labels = train_dataset.targets.numpy()
    test_images = test_dataset.data.numpy()
    test_labels = test_dataset.targets.numpy()
    
    # Reshape images and normalize
    train_images = train_images.reshape(-1, 1, 28, 28).astype(np.float32) / 255.0
    test_images = test_images.reshape(-1, 1, 28, 28).astype(np.float32) / 255.0
    
    # Create Arrow tables
    train_table = pa.Table.from_arrays([
        pa.array(train_images.tobytes()),
        pa.array(train_labels)
    ], names=['image', 'label'])
    
    test_table = pa.Table.from_arrays([
        pa.array(test_images.tobytes()),
        pa.array(test_labels)
    ], names=['image', 'label'])
    
    # Save as Parquet files
    pq.write_table(train_table, data_dir / 'train.parquet')
    pq.write_table(test_table, data_dir / 'test.parquet')

if __name__ == "__main__":
    prepare_mnist_data()
```

Run the data preparation script:
```bash
python prepare_mnist.py
```

## 3. Model Configuration

Create a model configuration file `mnist_config.json`:

```json
{
    "model_name": "mnist_cnn",
    "architecture": {
        "type": "cnn",
        "input_channels": 1,
        "conv_layers": [
            {"filters": 32, "kernel_size": 3, "stride": 1, "padding": 1},
            {"filters": 64, "kernel_size": 3, "stride": 1, "padding": 1}
        ],
        "fc_layers": [1024, 128, 10],
        "activation": "relu",
        "dropout": 0.5
    },
    "training": {
        "batch_size": 64,
        "learning_rate": 0.001,
        "epochs": 10,
        "optimizer": "adam"
    }
}
```

## 4. Training

Run the training with our secure training system:

```bash
./target/release/train \
    --model-name mnist_cnn \
    --data-path ./data/mnist \
    --checkpoint-dir ./checkpoints/mnist \
    --config ./mnist_config.json \
    --epochs 10 \
    --batch-size 64 \
    --learning-rate 0.001 \
    --device cuda
```

## 5. Monitor Training

### View Training Progress
```bash
tail -f /var/log/dcms/training.log
```

### Monitor GPU Usage (if using CUDA)
```bash
watch -n 1 nvidia-smi
```

### Access Metrics Dashboard
Open `http://localhost:3000` in your browser to view the Grafana dashboard.

## 6. Evaluate Results

After training completes, evaluate the model:

```bash
./target/release/evaluate \
    --model-path ./checkpoints/mnist/final_model.pt \
    --data-path ./data/mnist/test.parquet \
    --device cuda
```

Expected output:
```
Test Accuracy: ~98.5%
Test Loss: ~0.05
Confusion Matrix:
[...]
```

## 7. Model Deployment

### Export Model for Production

```bash
./target/release/export \
    --model-path ./checkpoints/mnist/final_model.pt \
    --output-path ./production/mnist_model \
    --format onnx
```

### Test Inference

Create a test script `test_inference.py`:

```python
import onnxruntime as ort
import numpy as np
from PIL import Image

def preprocess_image(image_path):
    # Load and preprocess image
    img = Image.open(image_path).convert('L')
    img = img.resize((28, 28))
    img_array = np.array(img, dtype=np.float32)
    img_array = img_array.reshape(1, 1, 28, 28) / 255.0
    return img_array

def run_inference(model_path, image_path):
    # Load ONNX model
    session = ort.InferenceSession(model_path)
    
    # Preprocess image
    input_data = preprocess_image(image_path)
    
    # Run inference
    input_name = session.get_inputs()[0].name
    output_name = session.get_outputs()[0].name
    result = session.run([output_name], {input_name: input_data})
    
    # Get prediction
    prediction = np.argmax(result[0])
    return prediction

if __name__ == "__main__":
    model_path = "./production/mnist_model/model.onnx"
    image_path = "./test_images/digit.png"
    prediction = run_inference(model_path, image_path)
    print(f"Predicted digit: {prediction}")
```

## 8. Performance Optimization

### Memory Usage
- If you encounter OOM errors, reduce batch size
- Monitor GPU memory usage with `nvidia-smi`
- Use CPU fallback if needed

### Training Speed
- Experiment with different batch sizes
- Enable CUDA optimization if available
- Use mixed precision training for faster computation

### Data Loading
- Use memory mapping for large datasets
- Enable data prefetching
- Optimize number of worker threads

## 9. Security Considerations

### Data Protection
- MNIST data is automatically encrypted at rest
- Training occurs within secure enclave
- Model parameters are protected

### Access Control
- Use appropriate IAM roles
- Monitor access logs
- Implement least privilege principle

### Model Security
- Checkpoints are encrypted
- Secure parameter updates
- Protected inference endpoints

## 10. Troubleshooting

### Common Issues

1. **Data Loading Errors**
   ```
   Error: Failed to load MNIST data
   ```
   - Verify data preparation completed successfully
   - Check file permissions
   - Ensure correct data path

2. **CUDA Errors**
   ```
   Error: CUDA not available
   ```
   - Install NVIDIA drivers
   - Check GPU compatibility
   - Verify CUDA toolkit installation

3. **Memory Issues**
   ```
   Error: Out of memory
   ```
   - Reduce batch size
   - Free unused memory
   - Check memory leaks

## 11. Next Steps

1. **Experiment with Architecture**
   - Try different network architectures
   - Adjust hyperparameters
   - Implement data augmentation

2. **Production Deployment**
   - Set up monitoring
   - Configure auto-scaling
   - Implement A/B testing

3. **Advanced Features**
   - Implement transfer learning
   - Add model versioning
   - Enable distributed training

## Support

For issues or questions:
- Check the [User Guide](../USER_GUIDE.md)
- Open GitHub issues
- Contact support team 