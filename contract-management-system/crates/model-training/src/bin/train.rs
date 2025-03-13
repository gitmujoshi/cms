// External dependencies
use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;
use model_training::training::pipeline::{TrainingConfig, TrainingPipeline};
use tracing_subscriber::{fmt, EnvFilter};

/// Command-line interface for the secure model training system
/// This structure defines all available CLI arguments and their descriptions
#[derive(Parser)]
#[command(author, version, about = "Secure AI Model Training CLI")]
struct Cli {
    /// Name of the model architecture to train (e.g., "resnet50", "bert-base")
    /// This determines the neural network architecture that will be used
    #[arg(short, long)]
    model_name: String,

    /// Path to the directory containing the training data
    /// The data should be in a format compatible with the preprocessing pipeline
    #[arg(short, long)]
    data_path: PathBuf,

    /// Directory where model checkpoints will be saved
    /// Checkpoints are encrypted and saved here every few epochs
    #[arg(short, long)]
    checkpoint_dir: PathBuf,

    /// Number of complete passes through the training dataset
    /// Higher values allow more learning but increase training time
    #[arg(short, long, default_value = "10")]
    epochs: usize,

    /// Number of samples to process in each training iteration
    /// Larger batches use more memory but can train faster
    #[arg(short, long, default_value = "32")]
    batch_size: usize,

    /// Rate at which the model learns from the data
    /// Higher values can learn faster but might be less stable
    #[arg(short, long, default_value = "0.001")]
    learning_rate: f64,

    /// Computation device to use for training
    /// Use "cuda" for GPU acceleration if available
    #[arg(short, long, default_value = "cpu")]
    device: String,
}

/// Main entry point for the training CLI
/// This function:
/// 1. Sets up logging
/// 2. Parses command line arguments
/// 3. Configures and initializes the training pipeline
/// 4. Executes the training process
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging with environment-based filter
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Parse command line arguments into structured config
    let cli = Cli::parse();

    // Create training configuration from CLI arguments
    let config = TrainingConfig {
        model_name: cli.model_name,
        batch_size: cli.batch_size,
        learning_rate: cli.learning_rate,
        epochs: cli.epochs,
        device: cli.device,
        checkpoint_dir: cli.checkpoint_dir,
        data_path: cli.data_path,
    };

    // Initialize the training pipeline with the configuration
    let mut pipeline = TrainingPipeline::new(config).await?;
    
    // Start the training process
    pipeline.train().await?;

    Ok(())
} 