use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::process::Command as AsyncCommand;
use tracing::{info, warn};
use uuid::Uuid;

/// Configuration for secure storage
#[derive(Debug, Clone)]
pub struct SecureStorageConfig {
    /// Path to the encrypted container file
    pub container_path: PathBuf,
    /// Size of the container in MB
    pub container_size_mb: u64,
    /// Mount point for the decrypted filesystem
    pub mount_point: PathBuf,
    /// Cipher specification for LUKS
    pub cipher: String,
    /// Key size in bits
    pub key_size: u32,
}

impl Default for SecureStorageConfig {
    fn default() -> Self {
        Self {
            container_path: PathBuf::from("/tmp/secure_container"),
            container_size_mb: 1024,
            mount_point: PathBuf::from("/mnt/secure"),
            cipher: String::from("aes-xts-plain64"),
            key_size: 512,
        }
    }
}

/// Manages secure storage using LUKS and FUSE
pub struct SecureStorage {
    config: SecureStorageConfig,
    device_name: String,
    is_mounted: bool,
}

impl SecureStorage {
    /// Creates a new secure storage instance
    pub fn new(config: SecureStorageConfig) -> Self {
        Self {
            config,
            device_name: format!("secure_{}", Uuid::new_v4().simple()),
            is_mounted: false,
        }
    }

    /// Initializes the secure storage container
    pub async fn initialize(&self) -> Result<()> {
        // Create container file
        info!("Creating container file at {:?}", self.config.container_path);
        AsyncCommand::new("dd")
            .args(&[
                "if=/dev/zero",
                &format!("of={}", self.config.container_path.display()),
                "bs=1M",
                &format!("count={}", self.config.container_size_mb),
            ])
            .output()
            .await
            .context("Failed to create container file")?;

        // Initialize LUKS container
        info!("Initializing LUKS container");
        AsyncCommand::new("cryptsetup")
            .args(&[
                "luksFormat",
                "--type=luks2",
                &format!("--cipher={}", self.config.cipher),
                &format!("--key-size={}", self.config.key_size),
                &self.config.container_path.to_string_lossy(),
            ])
            .output()
            .await
            .context("Failed to initialize LUKS container")?;

        Ok(())
    }

    /// Opens the LUKS container and mounts it
    pub async fn mount(&mut self) -> Result<()> {
        if self.is_mounted {
            warn!("Storage is already mounted");
            return Ok(());
        }

        // Open LUKS container
        info!("Opening LUKS container");
        AsyncCommand::new("cryptsetup")
            .args(&[
                "open",
                &self.config.container_path.to_string_lossy(),
                &self.device_name,
            ])
            .output()
            .await
            .context("Failed to open LUKS container")?;

        // Create filesystem
        info!("Creating ext4 filesystem");
        AsyncCommand::new("mkfs.ext4")
            .arg(format!("/dev/mapper/{}", self.device_name))
            .output()
            .await
            .context("Failed to create filesystem")?;

        // Create mount point
        std::fs::create_dir_all(&self.config.mount_point)
            .context("Failed to create mount point")?;

        // Mount filesystem
        info!("Mounting filesystem");
        AsyncCommand::new("mount")
            .args(&[
                format!("/dev/mapper/{}", self.device_name),
                self.config.mount_point.to_string_lossy().into_owned(),
            ])
            .output()
            .await
            .context("Failed to mount filesystem")?;

        self.is_mounted = true;
        Ok(())
    }

    /// Unmounts the filesystem and closes the LUKS container
    pub async fn unmount(&mut self) -> Result<()> {
        if !self.is_mounted {
            warn!("Storage is not mounted");
            return Ok(());
        }

        // Unmount filesystem
        info!("Unmounting filesystem");
        AsyncCommand::new("umount")
            .arg(&self.config.mount_point)
            .output()
            .await
            .context("Failed to unmount filesystem")?;

        // Close LUKS container
        info!("Closing LUKS container");
        AsyncCommand::new("cryptsetup")
            .args(&["close", &self.device_name])
            .output()
            .await
            .context("Failed to close LUKS container")?;

        self.is_mounted = false;
        Ok(())
    }

    /// Copies data into the secure storage
    pub async fn copy_data(&self, source_path: &Path) -> Result<()> {
        if !self.is_mounted {
            return Err(anyhow::anyhow!("Storage is not mounted"));
        }

        info!("Copying data to secure storage");
        AsyncCommand::new("cp")
            .args(&[
                "-r",
                &source_path.to_string_lossy(),
                &self.config.mount_point.to_string_lossy(),
            ])
            .output()
            .await
            .context("Failed to copy data")?;

        Ok(())
    }

    /// Returns the path where data is mounted
    pub fn get_mount_path(&self) -> &Path {
        &self.config.mount_point
    }
}

impl Drop for SecureStorage {
    fn drop(&mut self) {
        if self.is_mounted {
            // Use blocking command in drop since async isn't available
            if let Err(e) = Command::new("umount")
                .arg(&self.config.mount_point)
                .output()
            {
                eprintln!("Error unmounting secure storage: {}", e);
            }
            
            if let Err(e) = Command::new("cryptsetup")
                .args(&["close", &self.device_name])
                .output()
            {
                eprintln!("Error closing LUKS container: {}", e);
            }
            
            // Attempt to remove the container file
            if let Err(e) = std::fs::remove_file(&self.config.container_path) {
                eprintln!("Error removing container file: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_secure_storage_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let config = SecureStorageConfig {
            container_path: temp_dir.path().join("container"),
            container_size_mb: 10,
            mount_point: temp_dir.path().join("mount"),
            ..Default::default()
        };

        let mut storage = SecureStorage::new(config);
        
        // Test initialization
        assert!(storage.initialize().await.is_ok());
        
        // Test mounting
        assert!(storage.mount().await.is_ok());
        assert!(storage.is_mounted);
        
        // Test unmounting
        assert!(storage.unmount().await.is_ok());
        assert!(!storage.is_mounted);
    }
} 