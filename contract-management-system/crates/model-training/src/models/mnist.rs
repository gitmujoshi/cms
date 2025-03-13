use anyhow::Result;
use tch::{nn, nn::ModuleT, Device, Tensor};
use serde::{Deserialize, Serialize};

/// Configuration for the MNIST CNN model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MnistConfig {
    /// Number of input channels (1 for grayscale MNIST)
    pub input_channels: i64,
    /// Configuration for convolutional layers
    pub conv_layers: Vec<ConvLayerConfig>,
    /// Configuration for fully connected layers
    pub fc_layers: Vec<i64>,
    /// Dropout probability
    pub dropout: f64,
}

/// Configuration for a convolutional layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvLayerConfig {
    pub filters: i64,
    pub kernel_size: i64,
    pub stride: i64,
    pub padding: i64,
}

/// MNIST CNN model implementation
pub struct MnistCnn {
    conv_layers: Vec<nn::Conv2D>,
    fc_layers: Vec<nn::Linear>,
    dropout: f64,
    device: Device,
}

impl MnistCnn {
    /// Creates a new MNIST CNN model
    pub fn new(vs: &nn::Path, config: &MnistConfig, device: Device) -> Result<Self> {
        let mut conv_layers = Vec::new();
        let mut fc_layers = Vec::new();
        
        // Input size tracking for FC layer
        let mut current_channels = config.input_channels;
        let mut current_size = 28; // MNIST image size
        
        // Build convolutional layers
        for conv_config in &config.conv_layers {
            let conv = nn::conv2d(
                vs,
                current_channels,
                conv_config.filters,
                conv_config.kernel_size,
                nn::ConvConfig {
                    stride: conv_config.stride,
                    padding: conv_config.padding,
                    ..Default::default()
                },
            );
            
            // Update dimensions for next layer
            current_channels = conv_config.filters;
            current_size = (current_size + 2 * conv_config.padding - conv_config.kernel_size) 
                / conv_config.stride + 1;
            
            conv_layers.push(conv);
        }
        
        // Calculate input size for first FC layer
        let fc_input_size = current_channels * current_size * current_size;
        
        // Build fully connected layers
        let mut current_size = fc_input_size;
        for &output_size in &config.fc_layers {
            let fc = nn::linear(vs, current_size, output_size, Default::default());
            current_size = output_size;
            fc_layers.push(fc);
        }
        
        Ok(Self {
            conv_layers,
            fc_layers,
            dropout: config.dropout,
            device,
        })
    }
    
    /// Forward pass of the model
    pub fn forward(&self, xs: &Tensor) -> Tensor {
        let mut x = xs.to_device(self.device);
        
        // Convolutional layers with ReLU and max pooling
        for conv in &self.conv_layers {
            x = x.apply(conv)
                .relu()
                .max_pool2d_default(2);
        }
        
        // Flatten for fully connected layers
        let batch_size = x.size()[0];
        x = x.view([batch_size, -1]);
        
        // Fully connected layers
        for (i, fc) in self.fc_layers.iter().enumerate() {
            x = x.apply(fc);
            
            // Apply ReLU and dropout to all but last layer
            if i < self.fc_layers.len() - 1 {
                x = x.relu();
                x = x.dropout(self.dropout, true);
            }
        }
        
        // Final output with log_softmax
        x.log_softmax(-1, tch::Kind::Float)
    }
    
    /// Calculate loss for a batch
    pub fn loss(&self, xs: &Tensor, ys: &Tensor) -> Tensor {
        let logits = self.forward(xs);
        let target = ys.to_device(self.device);
        logits.nll_loss(target)
    }
    
    /// Calculate accuracy for a batch
    pub fn accuracy(&self, xs: &Tensor, ys: &Tensor) -> f64 {
        let logits = self.forward(xs);
        let target = ys.to_device(self.device);
        let pred = logits.argmax(-1, false);
        let correct = pred.eq_tensor(&target).sum(tch::Kind::Float);
        f64::from(correct) / f64::from(target.size()[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tch::Tensor;
    
    fn create_test_config() -> MnistConfig {
        MnistConfig {
            input_channels: 1,
            conv_layers: vec![
                ConvLayerConfig {
                    filters: 32,
                    kernel_size: 3,
                    stride: 1,
                    padding: 1,
                },
                ConvLayerConfig {
                    filters: 64,
                    kernel_size: 3,
                    stride: 1,
                    padding: 1,
                },
            ],
            fc_layers: vec![1024, 128, 10],
            dropout: 0.5,
        }
    }
    
    #[test]
    fn test_model_creation() {
        let vs = nn::VarStore::new(Device::Cpu);
        let config = create_test_config();
        let model = MnistCnn::new(&vs.root(), &config, Device::Cpu);
        assert!(model.is_ok());
    }
    
    #[test]
    fn test_forward_pass() {
        let vs = nn::VarStore::new(Device::Cpu);
        let config = create_test_config();
        let model = MnistCnn::new(&vs.root(), &config, Device::Cpu).unwrap();
        
        // Create dummy batch
        let batch_size = 32;
        let xs = Tensor::zeros(&[batch_size, 1, 28, 28], (tch::Kind::Float, Device::Cpu));
        
        let output = model.forward(&xs);
        assert_eq!(output.size(), &[batch_size, 10]);
    }
    
    #[test]
    fn test_loss_calculation() {
        let vs = nn::VarStore::new(Device::Cpu);
        let config = create_test_config();
        let model = MnistCnn::new(&vs.root(), &config, Device::Cpu).unwrap();
        
        // Create dummy batch
        let batch_size = 32;
        let xs = Tensor::zeros(&[batch_size, 1, 28, 28], (tch::Kind::Float, Device::Cpu));
        let ys = Tensor::zeros(&[batch_size], (tch::Kind::Int64, Device::Cpu));
        
        let loss = model.loss(&xs, &ys);
        assert!(loss.dim() == 0);
    }
} 