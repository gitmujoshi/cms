use aws_sdk_s3::Client as S3Client;
use anyhow::{Result, Context};

pub struct S3Handler {
    client: S3Client,
    bucket: String,
}

impl S3Handler {
    pub async fn new(bucket: String) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = S3Client::new(&config);

        Ok(Self {
            client,
            bucket,
        })
    }

    pub async fn download_data(&self, key: &str) -> Result<Vec<u8>> {
        let response = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .context("Failed to download data from S3")?;

        let data = response
            .body
            .collect()
            .await
            .context("Failed to collect response body")?;

        Ok(data.to_vec())
    }

    pub async fn upload_data(&self, key: &str, data: &[u8]) -> Result<()> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(data.to_vec().into())
            .send()
            .await
            .context("Failed to upload data to S3")?;

        Ok(())
    }

    pub async fn list_objects(&self, prefix: &str) -> Result<Vec<String>> {
        let response = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .send()
            .await
            .context("Failed to list objects in S3")?;

        let keys = response
            .contents
            .unwrap_or_default()
            .into_iter()
            .filter_map(|obj| obj.key)
            .collect();

        Ok(keys)
    }
} 