use aws_sdk_kms::Client as KmsClient;
use aws_sdk_kms::types::Blob;
use anyhow::{Result, Context};
use aws_nitro_enclaves_sdk::attestation::AttestationDocument;

pub struct KmsHandler {
    client: KmsClient,
    key_id: String,
}

impl KmsHandler {
    pub async fn new(key_id: String) -> Result<Self> {
        let config = aws_config::load_from_env().await;
        let client = KmsClient::new(&config);

        Ok(Self {
            client,
            key_id,
        })
    }

    pub async fn decrypt_data(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        // Get attestation document
        let attestation_doc = AttestationDocument::new()
            .context("Failed to create attestation document")?;

        // Decrypt the data using KMS
        let decrypt_output = self.client
            .decrypt()
            .key_id(&self.key_id)
            .ciphertext_blob(Blob::new(encrypted_data))
            .encryption_algorithm(aws_sdk_kms::types::EncryptionAlgorithmSpec::SymmetricDefault)
            .recipient(aws_sdk_kms::types::RecipientInfo::builder()
                .attestation_document(Blob::new(attestation_doc.as_bytes()))
                .build()?)
            .send()
            .await
            .context("Failed to decrypt data")?;

        Ok(decrypt_output.plaintext()
            .context("No plaintext in response")?
            .as_ref()
            .to_vec())
    }

    pub async fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let encrypt_output = self.client
            .encrypt()
            .key_id(&self.key_id)
            .plaintext(Blob::new(plaintext))
            .encryption_algorithm(aws_sdk_kms::types::EncryptionAlgorithmSpec::SymmetricDefault)
            .send()
            .await
            .context("Failed to encrypt data")?;

        Ok(encrypt_output.ciphertext_blob()
            .context("No ciphertext in response")?
            .as_ref()
            .to_vec())
    }
} 