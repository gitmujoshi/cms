# Quick Start Guide: Secure AI Model Training

This guide provides a quick overview of how to get started with the Secure AI Model Training system.

## 1. Installation

```bash
# Clone the repository
git clone https://github.com/your-org/contract-management-system.git
cd contract-management-system

# Build the training binary
cd crates/model-training
cargo build --release
```

## 2. Prepare Your Data

1. Organize your data in the following structure:
```
data/
├── train/
│   ├── features.parquet
│   └── labels.parquet
└── validation/
    ├── features.parquet
    └── labels.parquet
```

2. Ensure data is properly formatted:
   - Normalized features
   - Encoded labels
   - Consistent schema

## 3. Configure Environment

```bash
# Set logging level
export RUST_LOG=info

# Configure AWS credentials
export AWS_ACCESS_KEY_ID=your_access_key
export AWS_SECRET_ACCESS_KEY=your_secret_key
export AWS_REGION=us-west-2

# Configure enclave resources
export ENCLAVE_CPU_COUNT=4
export ENCLAVE_MEMORY_MIB=8192
```

## 4. Run Training

Basic training command:
```bash
./target/release/train \
    --model-name "resnet50" \
    --data-path ./data \
    --checkpoint-dir ./checkpoints \
    --epochs 10 \
    --batch-size 32 \
    --device cuda
```

## 5. Monitor Progress

1. View training logs:
```bash
tail -f /var/log/dcms/training.log
```

2. Access metrics:
- Open `http://localhost:9090/metrics` for Prometheus metrics
- Open `http://localhost:3000` for Grafana dashboard

## 6. Common Operations

### Check Training Status
```bash
ps aux | grep train
```

### View GPU Usage
```bash
nvidia-smi
```

### Check Checkpoints
```bash
ls -l ./checkpoints
```

## 7. Troubleshooting

If you encounter issues:

1. Check logs:
```bash
cat /var/log/dcms/training.log
```

2. Verify GPU availability:
```bash
nvidia-smi
```

3. Check enclave status:
```bash
nitro-cli describe-enclaves
```

## 8. Next Steps

- Read the full [User Guide](USER_GUIDE.md) for detailed information
- Configure advanced training parameters
- Set up monitoring and alerting
- Implement production deployment

## 9. Support

For issues or questions:
- Open an issue on GitHub
- Contact support@your-org.com
- Check the troubleshooting guide in the documentation 