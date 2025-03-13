use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use arrow::array::{Array, PrimitiveArray};
use arrow::datatypes::Float32Type;
use parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use parquet::file::reader::SerializedFileReader;
use std::fs::File;
use tch::Tensor;
use tracing::info;

use crate::enclave::integration::EnclaveSession;
use crate::utils::secure_storage::{SecureStorage, SecureStorageConfig};

/// MNIST dataset preprocessor with secure storage
pub struct MnistPreprocessor {
    train_path: PathBuf,
    test_path: PathBuf,
    batch_size: usize,
    secure_storage: Option<SecureStorage>,
}

impl MnistPreprocessor {
    /// Creates a new MNIST preprocessor
    pub fn new(data_path: &Path, batch_size: usize) -> Result<Self> {
        let train_path = data_path.join("train.parquet");
        let test_path = data_path.join("test.parquet");
        
        // Validate that files exist
        if !train_path.exists() {
            return Err(anyhow::anyhow!("Training data file not found: {:?}", train_path));
        }
        if !test_path.exists() {
            return Err(anyhow::anyhow!("Test data file not found: {:?}", test_path));
        }
        
        // Validate batch size
        if batch_size == 0 {
            return Err(anyhow::anyhow!("Batch size must be greater than 0"));
        }
        
        Ok(Self {
            train_path,
            test_path,
            batch_size,
            secure_storage: None,
        })
    }
    
    /// Initializes secure storage for the dataset
    pub async fn initialize_secure_storage(&mut self, container_size_mb: u64) -> Result<()> {
        let config = SecureStorageConfig {
            container_path: PathBuf::from("/tmp/mnist_secure"),
            container_size_mb,
            mount_point: PathBuf::from("/mnt/mnist_secure"),
            ..Default::default()
        };
        
        let mut storage = SecureStorage::new(config);
        storage.initialize().await?;
        storage.mount().await?;
        
        // Copy dataset files to secure storage
        storage.copy_data(&self.train_path).await?;
        storage.copy_data(&self.test_path).await?;
        
        // Update paths to point to secure storage
        let mount_path = storage.get_mount_path().to_path_buf();
        self.train_path = mount_path.join(self.train_path.file_name().unwrap());
        self.test_path = mount_path.join(self.test_path.file_name().unwrap());
        
        self.secure_storage = Some(storage);
        Ok(())
    }
    
    /// Loads and preprocesses data within the secure enclave
    pub async fn prepare_data_in_enclave(
        &self,
        enclave_session: &EnclaveSession,
    ) -> Result<(MnistDataset, MnistDataset)> {
        info!("Loading data from secure storage within enclave");
        
        // Verify we're using secure storage
        if self.secure_storage.is_none() {
            return Err(anyhow::anyhow!("Secure storage not initialized"));
        }
        
        // Load data securely within enclave
        let train_data = self.load_dataset(&self.train_path, true).await?;
        let test_data = self.load_dataset(&self.test_path, false).await?;
        
        Ok((train_data, test_data))
    }
    
    /// Loads a dataset from a Parquet file
    async fn load_dataset(&self, path: &Path, is_training: bool) -> Result<MnistDataset> {
        info!("Loading dataset from {:?}", path);
        
        let file = File::open(path)
            .with_context(|| format!("Failed to open file: {:?}", path))?;
        let reader = SerializedFileReader::new(file)?;
        let mut arrow_reader = ParquetRecordBatchReader::try_new(reader, 1024)?;
        
        let mut images = Vec::new();
        let mut labels = Vec::new();
        
        // Read batches
        while let Some(batch) = arrow_reader.next() {
            let batch = batch?;
            
            // Validate column count
            if batch.num_columns() != 2 {
                return Err(anyhow::anyhow!(
                    "Invalid dataset format: expected 2 columns, found {}",
                    batch.num_columns()
                ));
            }
            
            // Extract and validate images
            if let Some(image_array) = batch
                .column(0)
                .as_any()
                .downcast_ref::<PrimitiveArray<Float32Type>>()
            {
                // Validate image dimensions
                if image_array.len() % 784 != 0 {  // 28x28 = 784
                    return Err(anyhow::anyhow!("Invalid image dimensions"));
                }
                images.extend(image_array.values().iter());
            } else {
                return Err(anyhow::anyhow!("Invalid image data type"));
            }
            
            // Extract and validate labels
            if let Some(label_array) = batch
                .column(1)
                .as_any()
                .downcast_ref::<PrimitiveArray<Float32Type>>()
            {
                // Validate label values
                for &label in label_array.values() {
                    if label < 0.0 || label > 9.0 {
                        return Err(anyhow::anyhow!("Invalid label value: {}", label));
                    }
                }
                labels.extend(label_array.values().iter());
            } else {
                return Err(anyhow::anyhow!("Invalid label data type"));
            }
        }
        
        // Validate dataset size
        if images.is_empty() || labels.is_empty() {
            return Err(anyhow::anyhow!("Empty dataset"));
        }
        if images.len() / 784 != labels.len() {
            return Err(anyhow::anyhow!("Mismatched number of images and labels"));
        }
        
        // Convert to tensors
        let images_tensor = Tensor::of_slice(&images)
            .view([-1, 1, 28, 28]);
        let labels_tensor = Tensor::of_slice(&labels)
            .to_kind(tch::Kind::Int64);
        
        Ok(MnistDataset {
            images: images_tensor,
            labels: labels_tensor,
            batch_size: self.batch_size,
            is_training,
        })
    }
}

impl Drop for MnistPreprocessor {
    fn drop(&mut self) {
        // Secure storage cleanup is handled by its own Drop implementation
        self.secure_storage.take();
    }
}

/// MNIST dataset wrapper
pub struct MnistDataset {
    images: Tensor,
    labels: Tensor,
    batch_size: usize,
    is_training: bool,
}

impl MnistDataset {
    /// Creates an iterator over batches
    pub fn batch_iter(&self) -> MnistBatchIterator {
        let num_samples = self.images.size()[0] as usize;
        
        MnistBatchIterator {
            dataset: self,
            current_idx: 0,
            indices: if self.is_training {
                Tensor::randperm(num_samples as i64, (tch::Kind::Int64, self.images.device()))
            } else {
                Tensor::arange(num_samples as i64, (tch::Kind::Int64, self.images.device()))
            },
        }
    }
}

/// Iterator over MNIST batches
pub struct MnistBatchIterator<'a> {
    dataset: &'a MnistDataset,
    current_idx: usize,
    indices: Tensor,
}

impl<'a> Iterator for MnistBatchIterator<'a> {
    type Item = (Tensor, Tensor);
    
    fn next(&mut self) -> Option<Self::Item> {
        let num_samples = self.dataset.images.size()[0] as usize;
        
        if self.current_idx >= num_samples {
            return None;
        }
        
        let end_idx = (self.current_idx + self.dataset.batch_size).min(num_samples);
        let batch_indices = self.indices.slice(0, self.current_idx as i64, end_idx as i64, 1);
        
        let batch_x = self.dataset.images.index_select(0, &batch_indices);
        let batch_y = self.dataset.labels.index_select(0, &batch_indices);
        
        self.current_idx = end_idx;
        
        Some((batch_x, batch_y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;
    use std::fs::{create_dir_all, File, write};
    use arrow::array::Float32Array;
    use arrow::record_batch::RecordBatch;
    use parquet::arrow::arrow_writer::ArrowWriter;
    use arrow::datatypes::{Schema, Field, DataType};
    
    async fn create_test_data() -> Result<PathBuf> {
        let test_dir = temp_dir().join("mnist_test");
        create_dir_all(&test_dir)?;
        
        // Create schema for test data
        let schema = Schema::new(vec![
            Field::new("image", DataType::Float32, false),
            Field::new("label", DataType::Float32, false),
        ]);
        
        // Create test data
        let image_data = vec![0.0f32; 784];  // One 28x28 image
        let label_data = vec![1.0f32];       // One label
        
        let image_array = Float32Array::from(image_data);
        let label_array = Float32Array::from(label_data);
        
        let batch = RecordBatch::try_new(
            std::sync::Arc::new(schema.clone()),
            vec![
                std::sync::Arc::new(image_array),
                std::sync::Arc::new(label_array),
            ],
        )?;
        
        // Write train data
        let train_file = File::create(test_dir.join("train.parquet"))?;
        let mut writer = ArrowWriter::try_new(train_file, std::sync::Arc::new(schema.clone()), None)?;
        writer.write(&batch)?;
        writer.close()?;
        
        // Write test data
        let test_file = File::create(test_dir.join("test.parquet"))?;
        let mut writer = ArrowWriter::try_new(test_file, std::sync::Arc::new(schema), None)?;
        writer.write(&batch)?;
        writer.close()?;
        
        Ok(test_dir)
    }
    
    #[tokio::test]
    async fn test_preprocessor_initialization() {
        let test_dir = create_test_data().await.unwrap();
        
        // Test valid initialization
        let preprocessor = MnistPreprocessor::new(&test_dir, 32);
        assert!(preprocessor.is_ok());
        
        // Test invalid batch size
        let preprocessor = MnistPreprocessor::new(&test_dir, 0);
        assert!(preprocessor.is_err());
        
        // Test non-existent directory
        let preprocessor = MnistPreprocessor::new(Path::new("/nonexistent"), 32);
        assert!(preprocessor.is_err());
    }
    
    #[tokio::test]
    async fn test_preprocessor_with_secure_storage() {
        let test_dir = create_test_data().await.unwrap();
        let mut preprocessor = MnistPreprocessor::new(&test_dir, 32).unwrap();
        
        // Test secure storage initialization
        assert!(preprocessor.initialize_secure_storage(10).await.is_ok());
        assert!(preprocessor.secure_storage.is_some());
        
        // Test data loading
        let enclave_session = EnclaveSession::initialize().await.unwrap();
        let result = preprocessor.prepare_data_in_enclave(&enclave_session).await;
        assert!(result.is_ok());
        
        let (train_data, test_data) = result.unwrap();
        
        // Verify dataset properties
        assert_eq!(train_data.images.size()[1], 1);  // Channel dimension
        assert_eq!(train_data.images.size()[2], 28); // Height
        assert_eq!(train_data.images.size()[3], 28); // Width
        assert_eq!(train_data.images.size()[0], train_data.labels.size()[0]); // Batch dimension
    }
    
    #[tokio::test]
    async fn test_batch_iterator() {
        let test_dir = create_test_data().await.unwrap();
        let mut preprocessor = MnistPreprocessor::new(&test_dir, 1).unwrap();
        
        // Initialize storage and load data
        preprocessor.initialize_secure_storage(10).await.unwrap();
        let enclave_session = EnclaveSession::initialize().await.unwrap();
        let (train_data, _) = preprocessor.prepare_data_in_enclave(&enclave_session).await.unwrap();
        
        // Test batch iteration
        let mut batch_iter = train_data.batch_iter();
        let first_batch = batch_iter.next();
        assert!(first_batch.is_some());
        
        let (images, labels) = first_batch.unwrap();
        assert_eq!(images.size(), &[1, 1, 28, 28]);
        assert_eq!(labels.size(), &[1]);
    }
} 