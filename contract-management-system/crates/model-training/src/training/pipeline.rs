// External dependencies
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tch::{Device, Tensor};
use tracing::{info, warn};
use serde_json;
use chrono;

// Internal module imports
use crate::models::architecture::ModelArchitecture;
use crate::data::preprocessing::DataPreprocessor;
use crate::enclave::integration::EnclaveSession;

/// Configuration for the training pipeline
/// This structure defines all parameters needed to set up and run model training
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    /// Name of the model architecture to use (e.g., "resnet50", "bert-base")
    pub model_name: String,
    /// Number of samples to process in each training iteration
    pub batch_size: usize,
    /// Rate at which the model learns from the data
    pub learning_rate: f64,
    /// Number of complete passes through the training dataset
    pub epochs: usize,
    /// Computation device to use ("cpu" or "cuda" for GPU)
    pub device: String,
    /// Directory where model checkpoints will be saved
    pub checkpoint_dir: PathBuf,
    /// Path to the directory containing training data
    pub data_path: PathBuf,
}

/// Main training pipeline that orchestrates the secure model training process
/// This struct coordinates data preprocessing, model training, and enclave operations
pub struct TrainingPipeline {
    /// Training configuration parameters
    config: TrainingConfig,
    /// The neural network model being trained
    model: ModelArchitecture,
    /// Data preprocessing pipeline
    preprocessor: DataPreprocessor,
    /// Secure enclave session for protected computation
    enclave_session: EnclaveSession,
}

impl TrainingPipeline {
    /// Creates a new training pipeline instance
    /// 
    /// # Arguments
    /// * `config` - Configuration parameters for the training process
    /// 
    /// # Returns
    /// * `Result<Self>` - A new pipeline instance or an error
    pub async fn new(config: TrainingConfig) -> Result<Self> {
        let device = match config.device.as_str() {
            "cuda" if tch::Cuda::is_available() => Device::Cuda(0),
            _ => {
                warn!("CUDA not available, falling back to CPU");
                Device::Cpu
            }
        };

        let model = ModelArchitecture::new(&config.model_name, device)?;
        
        // Initialize preprocessor with secure storage
        let mut preprocessor = DataPreprocessor::new(&config.data_path)?;
        
        // Calculate required storage size (with 20% buffer)
        let data_size = std::fs::metadata(&config.data_path)?.len() / (1024 * 1024);  // Convert to MB
        let storage_size = (data_size * 12) / 10;  // Add 20% buffer
        
        info!("Initializing secure storage with size {}MB", storage_size);
        preprocessor.initialize_secure_storage(storage_size).await?;
        
        let enclave_session = EnclaveSession::initialize().await?;

        Ok(Self {
            config,
            model,
            preprocessor,
            enclave_session,
        })
    }

    /// Runs the training process
    pub async fn train(&mut self) -> Result<()> {
        info!("Starting training process");
        
        // Prepare data in secure storage within enclave
        info!("Preparing data in secure environment");
        let (train_data, test_data) = self.preprocessor
            .prepare_data_in_enclave(&mut self.enclave_session)
            .await?;
            
        let mut best_accuracy = 0.0;
        
        for epoch in 0..self.config.epochs {
            info!("Starting epoch {}/{}", epoch + 1, self.config.epochs);
            
            // Training phase
            self.model.train();
            let mut epoch_loss = 0.0;
            let mut batches = 0;
            
            for (images, labels) in train_data.iter() {
                // Process batch within enclave
                let (loss, _) = self.enclave_session.process_training_batch(
                    &mut self.model,
                    &images,
                    &labels
                ).await?;
                
                epoch_loss += loss;
                batches += 1;
                
                if batches % 100 == 0 {
                    info!(
                        "Epoch {}/{}, Batch {}: Loss = {:.4}",
                        epoch + 1, self.config.epochs, batches, loss
                    );
                }
            }
            
            let avg_loss = epoch_loss / batches as f64;
            
            // Evaluation phase
            self.model.eval();
            let accuracy = self.evaluate(&test_data).await?;
            
            info!(
                "Epoch {}/{} - Avg Loss: {:.4}, Accuracy: {:.2}%",
                epoch + 1, self.config.epochs, avg_loss, accuracy * 100.0
            );
            
            // Save checkpoint if accuracy improved
            if accuracy > best_accuracy {
                best_accuracy = accuracy;
                self.save_checkpoint(epoch, avg_loss, accuracy).await?;
            }
        }
        
        info!("Training completed. Best accuracy: {:.2}%", best_accuracy * 100.0);
        Ok(())
    }

    /// Evaluates the model on test data
    async fn evaluate(&self, test_data: &MnistDataset) -> Result<f64> {
        let mut correct = 0;
        let mut total = 0;
        
        for (images, labels) in test_data.batch_iter() {
            let predictions = self.enclave_session
                .process_inference_batch(&self.model, &images)
                .await
                .context("Failed to process inference batch")?;
                
            let batch_correct = predictions
                .argmax(-1, false)
                .eq(&labels)
                .sum()
                .int64_value();
                
            correct += batch_correct as i64;
            total += labels.size()[0] as i64;
            
            // Free memory explicitly
            drop(predictions);
        }
        
        if total == 0 {
            return Err(anyhow::anyhow!("No test samples processed"));
        }
        
        Ok(correct as f64 / total as f64)
    }

    /// Saves an encrypted checkpoint of the model
    /// 
    /// # Arguments
    /// * `epoch` - The current epoch number
    /// * `avg_loss` - The average loss for the epoch
    /// * `accuracy` - The accuracy of the model on the validation dataset
    async fn save_checkpoint(&self, epoch: usize, avg_loss: f64, accuracy: f64) -> Result<()> {
        // Create checkpoint directory if it doesn't exist
        if !self.config.checkpoint_dir.exists() {
            std::fs::create_dir_all(&self.config.checkpoint_dir)
                .context("Failed to create checkpoint directory")?;
        }
        
        let checkpoint_path = self.config.checkpoint_dir
            .join(format!("checkpoint_epoch_{}.pt", epoch));
            
        // Save checkpoint metadata
        let metadata = serde_json::json!({
            "epoch": epoch,
            "avg_loss": avg_loss,
            "accuracy": accuracy,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "model_name": self.config.model_name,
            "batch_size": self.config.batch_size,
            "learning_rate": self.config.learning_rate
        });
        
        let metadata_path = checkpoint_path.with_extension("json");
        std::fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)
            .context("Failed to save checkpoint metadata")?;
        
        // Save encrypted model checkpoint
        self.enclave_session
            .save_encrypted_checkpoint(&self.model, &checkpoint_path)
            .await?;
            
        // Verify checkpoint was saved
        if !checkpoint_path.exists() {
            return Err(anyhow::anyhow!("Checkpoint file was not created"));
        }
        
        // Verify checkpoint can be loaded
        let verification = self.enclave_session
            .verify_checkpoint(&checkpoint_path)
            .await?;
            
        if !verification {
            // Remove invalid checkpoint
            std::fs::remove_file(&checkpoint_path)
                .context("Failed to remove invalid checkpoint")?;
            std::fs::remove_file(&metadata_path)
                .context("Failed to remove invalid checkpoint metadata")?;
            return Err(anyhow::anyhow!("Checkpoint verification failed"));
        }
        
        info!(
            "Saved checkpoint at epoch {} with accuracy {:.2}%",
            epoch,
            accuracy * 100.0
        );
        
        Ok(())
    }
}

/// Unit tests for the training pipeline
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use std::env::temp_dir;
    use std::fs::{create_dir_all, remove_dir_all};
    use anyhow::Context;

    /// Test fixture providing a default configuration
    #[fixture]
    fn test_config() -> TrainingConfig {
        let data_path = temp_dir().join("test_data");
        let checkpoint_dir = temp_dir().join("checkpoints");
        
        // Clean up any existing test directories
        let _ = remove_dir_all(&data_path);
        let _ = remove_dir_all(&checkpoint_dir);
        
        // Create fresh directories
        create_dir_all(&data_path).unwrap();
        create_dir_all(&checkpoint_dir).unwrap();
        
        TrainingConfig {
            model_name: "test_model".to_string(),
            batch_size: 32,
            learning_rate: 0.001,
            epochs: 2,
            device: "cpu".to_string(),
            checkpoint_dir,
            data_path,
        }
    }

    /// Tests that the pipeline can be initialized with valid configuration
    #[tokio::test]
    async fn test_pipeline_initialization() {
        let config = test_config();
        let pipeline = TrainingPipeline::new(config).await;
        assert!(pipeline.is_ok());
    }
    
    /// Tests pipeline initialization with invalid configuration
    #[tokio::test]
    async fn test_pipeline_initialization_invalid_config() {
        let mut config = test_config();
        
        // Test with non-existent data path
        config.data_path = PathBuf::from("/nonexistent");
        let result = TrainingPipeline::new(config.clone()).await;
        assert!(result.is_err());
        
        // Test with invalid batch size
        config.batch_size = 0;
        let result = TrainingPipeline::new(config.clone()).await;
        assert!(result.is_err());
    }
    
    /// Tests checkpoint saving and loading
    #[tokio::test]
    async fn test_checkpoint_operations() {
        let config = test_config();
        let pipeline = TrainingPipeline::new(config.clone()).await.unwrap();
        
        // Test saving checkpoint
        let result = pipeline.save_checkpoint(0, 0.5, 0.95).await;
        assert!(result.is_ok());
        
        // Verify checkpoint files exist
        let checkpoint_path = config.checkpoint_dir.join("checkpoint_epoch_0.pt");
        let metadata_path = checkpoint_path.with_extension("json");
        assert!(checkpoint_path.exists());
        assert!(metadata_path.exists());
        
        // Verify metadata content
        let metadata = std::fs::read_to_string(metadata_path)
            .context("Failed to read checkpoint metadata")
            .unwrap();
        let json: serde_json::Value = serde_json::from_str(&metadata).unwrap();
        
        assert_eq!(json["epoch"], 0);
        assert_eq!(json["avg_loss"], 0.5);
        assert_eq!(json["accuracy"], 0.95);
        assert_eq!(json["model_name"], "test_model");
    }
    
    /// Tests cleanup after pipeline is dropped
    #[tokio::test]
    async fn test_pipeline_cleanup() {
        let config = test_config();
        let data_path = config.data_path.clone();
        
        {
            let pipeline = TrainingPipeline::new(config).await.unwrap();
            // Pipeline is dropped here
        }
        
        // Verify secure storage is cleaned up
        assert!(!data_path.join("secure_storage").exists());
    }
} 