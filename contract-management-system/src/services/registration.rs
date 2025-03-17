use std::sync::Arc;
use sea_orm::*;
use uuid::Uuid;

use crate::auth::did::DIDVerifier;
use crate::models::registration::{self, Model as Registration, PartyCredentials, RegistrationStatus};
use crate::models::contract::PartyRole;
use crate::error::{Result, ServiceError};

pub struct RegistrationService {
    db: DatabaseConnection,
    did_verifier: Arc<DIDVerifier>,
}

impl RegistrationService {
    pub fn new(db: DatabaseConnection, did_verifier: Arc<DIDVerifier>) -> Self {
        Self { db, did_verifier }
    }

    /// Submits a new party registration
    pub async fn submit_registration(
        &self,
        did: String,
        role: PartyRole,
        organization_name: String,
        description: String,
        contact_email: String,
        credentials: PartyCredentials,
    ) -> Result<Registration> {
        // Verify DID ownership
        self.verify_did_ownership(&did).await?;

        // Create registration record
        let registration = Registration::new(
            did,
            role,
            organization_name,
            description,
            contact_email,
            credentials,
        );

        // Save to database
        let model: registration::ActiveModel = registration.clone().into();
        registration::Entity::insert(model)
            .exec(&self.db)
            .await?;

        Ok(registration)
    }

    /// Verifies DID ownership through a challenge-response process
    async fn verify_did_ownership(&self, did: &str) -> Result<()> {
        // Generate challenge
        let challenge = self.did_verifier.generate_challenge(did).await
            .map_err(|e| ServiceError::ValidationError(format!("Failed to generate challenge: {}", e)))?;

        // In a real implementation, we would:
        // 1. Store the challenge
        // 2. Send it to the party
        // 3. Wait for their signed response
        // 4. Verify the signature
        
        Ok(())
    }

    /// Verifies registration credentials based on role
    async fn verify_credentials(&self, registration: &Registration) -> Result<bool> {
        match registration.role {
            PartyRole::DataProvider => self.verify_data_provider_credentials(&registration.credentials).await,
            PartyRole::DataConsumer => self.verify_data_consumer_credentials(&registration.credentials).await,
            PartyRole::InfrastructureProvider => self.verify_infrastructure_provider_credentials(&registration.credentials).await,
        }
    }

    async fn verify_data_provider_credentials(&self, credentials: &PartyCredentials) -> Result<bool> {
        // Verify data ownership proofs
        if let Some(proofs) = &credentials.data_ownership_proofs {
            if proofs.is_empty() {
                return Ok(false);
            }
            // Verify each proof
            for proof in proofs {
                // Implement proof verification logic
            }
        } else {
            return Ok(false);
        }

        // Verify required certifications
        if !credentials.certifications.iter().any(|c| c == "ISO27001") {
            return Ok(false);
        }

        // Verify compliance standards
        if !credentials.compliance_standards.iter().any(|c| c == "GDPR") {
            return Ok(false);
        }

        Ok(true)
    }

    async fn verify_data_consumer_credentials(&self, credentials: &PartyCredentials) -> Result<bool> {
        // Verify processing capabilities
        if let Some(capabilities) = &credentials.processing_capabilities {
            if capabilities.is_empty() {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        // Verify required certifications
        if !credentials.certifications.iter().any(|c| c == "ISO27001") {
            return Ok(false);
        }

        Ok(true)
    }

    async fn verify_infrastructure_provider_credentials(&self, credentials: &PartyCredentials) -> Result<bool> {
        // Verify infrastructure capabilities
        if let Some(capabilities) = &credentials.infrastructure_capabilities {
            if capabilities.is_empty() {
                return Ok(false);
            }
            // Verify each capability
            if !capabilities.iter().any(|c| c.contains("AWS Nitro")) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }

        // Verify required certifications
        if !credentials.certifications.iter().any(|c| c == "ISO27001") {
            return Ok(false);
        }

        Ok(true)
    }

    /// Reviews and approves/rejects a registration
    pub async fn review_registration(
        &self,
        registration_id: Uuid,
        approve: bool,
        notes: Option<String>,
    ) -> Result<Registration> {
        // Get registration
        let mut registration = registration::Entity::find_by_id(registration_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Registration not found".to_string()))?;

        // Verify credentials if approving
        if approve {
            let credentials_valid = self.verify_credentials(&registration).await?;
            if !credentials_valid {
                registration.update_status(
                    RegistrationStatus::Rejected,
                    Some("Invalid credentials".to_string()),
                );
                return Ok(registration);
            }
        }

        // Update status
        registration.update_status(
            if approve {
                RegistrationStatus::Active
            } else {
                RegistrationStatus::Rejected
            },
            notes,
        );

        // Save to database
        let model: registration::ActiveModel = registration.clone().into();
        registration::Entity::update(model)
            .exec(&self.db)
            .await?;

        Ok(registration)
    }

    /// Gets a registration by ID
    pub async fn get_registration(&self, id: Uuid) -> Result<Registration> {
        registration::Entity::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| ServiceError::NotFound("Registration not found".to_string()))
    }

    /// Lists all registrations with optional filters
    pub async fn list_registrations(
        &self,
        role: Option<PartyRole>,
        status: Option<RegistrationStatus>,
    ) -> Result<Vec<Registration>> {
        let mut query = registration::Entity::find();

        if let Some(role) = role {
            query = query.filter(registration::Column::Role.eq(role));
        }

        if let Some(status) = status {
            query = query.filter(registration::Column::Status.eq(status));
        }

        let registrations = query
            .order_by_desc(registration::Column::SubmittedAt)
            .all(&self.db)
            .await?;

        Ok(registrations)
    }

    /// Updates registration credentials
    pub async fn update_credentials(
        &self,
        registration_id: Uuid,
        credentials: PartyCredentials,
    ) -> Result<Registration> {
        let mut registration = self.get_registration(registration_id).await?;

        // Update credentials
        registration.update_credentials(credentials);

        // Save to database
        let model: registration::ActiveModel = registration.clone().into();
        registration::Entity::update(model)
            .exec(&self.db)
            .await?;

        Ok(registration)
    }

    /// Suspends a registration
    pub async fn suspend_registration(
        &self,
        registration_id: Uuid,
        reason: String,
    ) -> Result<Registration> {
        let mut registration = self.get_registration(registration_id).await?;

        registration.update_status(RegistrationStatus::Suspended, Some(reason));

        // Save to database
        let model: registration::ActiveModel = registration.clone().into();
        registration::Entity::update(model)
            .exec(&self.db)
            .await?;

        Ok(registration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::DatabaseBackend;
    use sea_orm::MockDatabase;

    fn create_test_credentials() -> PartyCredentials {
        PartyCredentials {
            certifications: vec!["ISO27001".to_string()],
            compliance_standards: vec!["GDPR".to_string()],
            data_ownership_proofs: Some(vec!["proof1".to_string()]),
            infrastructure_capabilities: Some(vec!["AWS Nitro".to_string()]),
            processing_capabilities: Some(vec!["ML Training".to_string()]),
        }
    }

    #[tokio::test]
    async fn test_registration_submission() {
        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![Registration::new(
                "did:example:123".to_string(),
                PartyRole::DataProvider,
                "Test Org".to_string(),
                "Test Description".to_string(),
                "test@example.com".to_string(),
                create_test_credentials(),
            )]])
            .into_connection();

        let did_verifier = Arc::new(DIDVerifier::new(Arc::new(crate::auth::did::MultiResolver::new(
            chrono::Duration::minutes(5),
        ))));

        let service = RegistrationService::new(db, did_verifier);

        let result = service.submit_registration(
            "did:example:123".to_string(),
            PartyRole::DataProvider,
            "Test Org".to_string(),
            "Test Description".to_string(),
            "test@example.com".to_string(),
            create_test_credentials(),
        ).await;

        assert!(result.is_ok());
        let registration = result.unwrap();
        assert_eq!(registration.status, RegistrationStatus::Pending);
    }

    #[tokio::test]
    async fn test_registration_approval() {
        let mut registration = Registration::new(
            "did:example:123".to_string(),
            PartyRole::DataProvider,
            "Test Org".to_string(),
            "Test Description".to_string(),
            "test@example.com".to_string(),
            create_test_credentials(),
        );

        let db = MockDatabase::new(DatabaseBackend::Postgres)
            .append_query_results(vec![vec![registration.clone()]])
            .append_exec_results(vec![Ok(1)])
            .into_connection();

        let did_verifier = Arc::new(DIDVerifier::new(Arc::new(crate::auth::did::MultiResolver::new(
            chrono::Duration::minutes(5),
        ))));

        let service = RegistrationService::new(db, did_verifier);

        let result = service.review_registration(
            registration.id,
            true,
            Some("Approved".to_string()),
        ).await;

        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.status, RegistrationStatus::Active);
        assert!(updated.verified_at.is_some());
    }
} 