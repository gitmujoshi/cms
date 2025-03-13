# DCMS Model Training

A secure machine learning training system for the Digital Contract Management System (DCMS).

## Features

- Secure model training using hardware enclaves
- Encrypted data storage with LUKS
- MNIST dataset support with Parquet format
- Checkpoint management and model versioning
- Flexible CPU/GPU execution support
- Comprehensive testing suite

## Quick Start

### Prerequisites

#### Minimum Requirements (CPU-only)
```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
    cryptsetup \
    build-essential \
    pkg-config \
    libssl-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Optional: GPU Support
```bash
# Install CUDA toolkit (if using GPU)
sudo apt-get install -y cuda-toolkit-11-8
```

### Building

```bash
# CPU-only build
cargo build --release

# With GPU support
cargo build --release --features cuda
```

### Running Tests

```bash
cargo test
```

### Training a Model

1. Prepare your data:
```bash
python scripts/prepare_mnist.py --output data/mnist
```

2. Configure training:
```bash
cp config/training_config.example.json config/training_config.json
# Edit config/training_config.json as needed
```

3. Start training:
```bash
# CPU training
cargo run --release -- train \
    --config config/training_config.json \
    --data-path data/mnist \
    --checkpoint-dir checkpoints \
    --device cpu

# GPU training (if available)
cargo run --release -- train \
    --config config/training_config.json \
    --data-path data/mnist \
    --checkpoint-dir checkpoints \
    --device cuda
```

## Configuration

### Training Configuration

```json
{
    "model_name": "mnist_cnn",
    "batch_size": 64,
    "learning_rate": 0.001,
    "epochs": 10,
    "device": "cpu",  // or "cuda" for GPU
    "checkpoint_dir": "checkpoints",
    "data_path": "data/mnist"
}
```

### Performance Guidelines

#### CPU Training
- Recommended for datasets up to 50,000 samples
- Batch size: 32-64 for optimal memory usage
- Multiple CPU cores utilized automatically
- Lower memory requirements

#### GPU Training
- Recommended for larger datasets
- Batch size: 128-512 for optimal GPU utilization
- Requires CUDA-capable GPU
- Higher memory requirements but faster training

### Security Configuration

```json
{
    "enclave": {
        "heap_size": "8G",
        "stack_size": "2M",
        "thread_count": 4
    },
    "storage": {
        "cipher": "aes-xts-plain64",
        "key_size": 512
    }
}
```

## Architecture

See [Architecture Documentation](../../docs/architecture/model_training.md) for detailed design information.

## Security

- All training data is encrypted at rest using LUKS
- Model training occurs within hardware enclaves
- Secure cleanup of sensitive data
- Regular security audits and updates

## Contributing

1. Fork the repository
2. Create your feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details. 