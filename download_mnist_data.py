import tensorflow as tf
import numpy as np
import matplotlib.pyplot as plt
import os
from datetime import datetime

def download_and_save_mnist():
    # Create data directory if it doesn't exist
    data_dir = "mnist_data"
    os.makedirs(data_dir, exist_ok=True)
    
    # Download MNIST data
    print("Downloading MNIST dataset...")
    (x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()
    
    # Save data as numpy arrays
    print("Saving data to files...")
    np.save(os.path.join(data_dir, 'x_train.npy'), x_train)
    np.save(os.path.join(data_dir, 'y_train.npy'), y_train)
    np.save(os.path.join(data_dir, 'x_test.npy'), x_test)
    np.save(os.path.join(data_dir, 'y_test.npy'), y_test)
    
    # Create a summary file
    summary_file = os.path.join(data_dir, 'dataset_summary.txt')
    with open(summary_file, 'w') as f:
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
    
    # Create visualization of sample images
    print("Creating sample visualizations...")
    plt.figure(figsize=(15, 5))
    
    # Plot 10 sample images from training set
    for i in range(10):
        plt.subplot(2, 5, i+1)
        plt.imshow(x_train[i], cmap='gray')
        plt.title(f'Digit: {y_train[i]}')
        plt.axis('off')
    
    plt.tight_layout()
    plt.savefig(os.path.join(data_dir, 'sample_images.png'))
    
    print(f"\nData saved to {data_dir}/")
    print("Files created:")
    print("- x_train.npy: Training images")
    print("- y_train.npy: Training labels")
    print("- x_test.npy: Test images")
    print("- y_test.npy: Test labels")
    print("- dataset_summary.txt: Dataset statistics")
    print("- sample_images.png: Visualization of sample images")

if __name__ == "__main__":
    download_and_save_mnist() 