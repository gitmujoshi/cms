mod kms;
mod s3;
mod mnist;

use anyhow::{Result, Context};
use log::{info, error};
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting enclave runtime");

    // Get configuration from environment variables
    let bucket = env::var("S3_BUCKET").context("S3_BUCKET not set")?;
    let key_id = env::var("KMS_KEY_ID").context("KMS_KEY_ID not set")?;
    let data_key = env::var("MNIST_DATA_KEY").context("MNIST_DATA_KEY not set")?;

    // Initialize handlers
    let s3_handler = s3::S3Handler::new(bucket.clone()).await?;
    let kms_handler = kms::KmsHandler::new(key_id).await?;

    // Download encrypted MNIST data
    info!("Downloading encrypted MNIST data");
    let encrypted_data = s3_handler.download_data(&data_key).await?;

    // Decrypt the data
    info!("Decrypting MNIST data");
    let decrypted_data = kms_handler.decrypt_data(&encrypted_data).await?;

    // Train the model
    info!("Starting model training");
    if let Err(e) = mnist::train_mnist(&decrypted_data).await {
        error!("Training failed: {}", e);
        return Err(e.into());
    }

    info!("Training completed successfully");
    Ok(())
} 