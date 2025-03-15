# Quick Start Guide - Model Training

## Overview

This guide provides quick steps to get started with model training in the Digital Contract Management System.

## Prerequisites

1. **AWS Account Setup**:
   ```bash
   aws configure
   ```

2. **Required Tools**:
   - Docker
   - Terraform >= 1.0.0
   - AWS CLI >= 2.0.0
   - Rust toolchain

## Quick Setup

1. **Clone and Build**:
   ```bash
   git clone <repository-url>
   cd contract-management-system
   cargo build --release
   ```

2. **Deploy Infrastructure**:
   ```bash
   cd deployment/terraform
   terraform init
   terraform apply
   ```

3. **Configure Environment**:
   ```bash
   export DATABASE_URL="postgresql://$(terraform output -raw rds_endpoint)"
   export AWS_REGION="us-west-2"
   ```

## Training a Model

1. **Prepare Data**:
   ```bash
   # Example with MNIST dataset
   cargo run --bin data-prep -- --dataset mnist --output-dir ./data
   ```

2. **Start Training**:
   ```bash
   cargo run --bin train -- \
     --model-name "my_model" \
     --dataset ./data \
     --epochs 10 \
     --batch-size 64
   ```

3. **Monitor Progress**:
   - Watch training metrics:
     ```bash
     aws logs tail /ecs/contract-management --follow
     ```
   - Monitor loss values in CloudWatch

## Understanding Loss

The loss value indicates model performance:
- Lower loss = better performance
- Watch for:
  - Steady decrease during training
  - Plateau indicating convergence
  - Sudden spikes suggesting issues

Example loss curve:
```
Epoch 1: Loss = 2.3024
Epoch 2: Loss = 1.8721
Epoch 3: Loss = 1.5432
...
```

## Common Commands

1. **Check Model Status**:
   ```bash
   cargo run --bin status -- --model-name "my_model"
   ```

2. **View Training Metrics**:
   ```bash
   cargo run --bin metrics -- --model-name "my_model"
   ```

3. **Stop Training**:
   ```bash
   cargo run --bin stop -- --model-name "my_model"
   ```

## Troubleshooting

1. **High Loss Values**:
   - Check data preprocessing
   - Verify model configuration
   - Adjust learning rate

2. **Deployment Issues**:
   - Verify AWS credentials
   - Check resource limits
   - Review security groups

## Next Steps

1. **Advanced Features**:
   - Custom loss functions
   - Model optimization
   - Distributed training

2. **Infrastructure**:
   - Scaling configuration
   - Monitoring setup
   - Backup strategies

For detailed information, see:
- [Technical Reference](TECHNICAL_REFERENCE.md)
- [User Guide](USER_GUIDE.md)
- [Infrastructure Guide](../deployment/README.md)

# Quick Start Guide: Privacy-Preserving Model Training

## Privacy-Preserving Training

### Prerequisites
- AWS account with Nitro Enclaves enabled
- Rust toolchain (1.70+)
- Docker for enclave deployment
- Training dataset prepared

### Quick Setup

1. **Configure Privacy Settings**
   ```toml
   # config/privacy.toml
   [differential_privacy]
   epsilon = 1.0                    # Privacy budget
   delta = 1e-5                    # Privacy relaxation
   noise_mechanism = "Gaussian"     # or "Laplace"
   max_grad_norm = 1.0             # Gradient clipping threshold
   ```

2. **Initialize Secure Environment**
   ```bash
   # Start Nitro Enclave
   cargo run --bin enclave-init
   
   # Verify attestation
   cargo run --bin verify-attestation
   ```

3. **Prepare Private Dataset**
   ```bash
   # Encrypt and load data into secure storage
   cargo run --bin prepare-private-data -- \
     --input-path /path/to/data \
     --encryption-key $KEY_PATH
   ```

### Start Training

1. **Launch Private Training**
   ```bash
   cargo run --bin train-private -- \
     --config config/privacy.toml \
     --model resnet50 \
     --batch-size 32
   ```

2. **Monitor Privacy Budget**
   ```bash
   # View real-time privacy metrics
   cargo run --bin monitor-privacy
   ```

### Common Commands

1. **Check Privacy Status**
   ```bash
   # View current privacy budget consumption
   cargo run --bin privacy-status
   ```

2. **View Training Metrics**
   ```bash
   # Monitor loss and accuracy with privacy guarantees
   cargo run --bin view-metrics --privacy
   ```

3. **Stop Training**
   ```bash
   # Gracefully stop training (preserves privacy budget)
   cargo run --bin stop-training
   ```

### Troubleshooting

#### High Privacy Budget Consumption
- Reduce batch size
- Increase noise scale
- Adjust gradient clipping threshold

#### Poor Model Performance
- Increase privacy budget (ε)
- Tune model architecture
- Adjust learning rate

#### Memory Issues
- Reduce batch size
- Enable gradient checkpointing
- Monitor enclave memory usage

### Best Practices

1. **Privacy Budget Management**
   - Start with conservative privacy settings
   - Monitor budget consumption rate
   - Use early stopping when needed

2. **Data Preparation**
   - Remove sensitive identifiers
   - Normalize data ranges
   - Use secure preprocessing

3. **Model Training**
   - Enable gradient clipping
   - Use appropriate noise mechanism
   - Monitor privacy metrics

### Next Steps

1. **Advanced Configuration**
   - Custom noise mechanisms
   - Advanced privacy accounting
   - Multi-party computation

2. **Production Deployment**
   - Automated privacy monitoring
   - Secure model serving
   - Compliance reporting

3. **Further Reading**
   - Technical Reference Manual
   - Privacy Guarantees Documentation
   - Security Architecture Guide 