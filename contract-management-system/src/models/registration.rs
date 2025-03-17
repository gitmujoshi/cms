use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sea_orm::entity::prelude::*;

use crate::models::contract::PartyRole;

/// Registration status for a party
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RegistrationStatus {
    /// Registration submitted but not verified
    Pending,
    /// Registration approved and active
    Active,
    /// Registration rejected
    Rejected,
    /// Registration suspended
    Suspended,
}

/// Represents the credentials and certifications of a party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartyCredentials {
    /// List of certifications held by the party
    pub certifications: Vec<String>,
    /// Compliance standards met
    pub compliance_standards: Vec<String>,
    /// Data ownership proofs (for Data Providers)
    pub data_ownership_proofs: Option<Vec<String>>,
    /// Infrastructure capabilities (for Infrastructure Providers)
    pub infrastructure_capabilities: Option<Vec<String>>,
    /// Processing capabilities (for Data Consumers)
    pub processing_capabilities: Option<Vec<String>>,
}

/// Main registration model for parties
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "party_registrations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    /// Decentralized Identifier of the party
    pub did: String,
    /// Role of the party in the system
    pub role: PartyRole,
    /// Organization or entity name
    pub organization_name: String,
    /// Organization description
    pub description: String,
    /// Contact email
    pub contact_email: String,
    /// Registration status
    pub status: RegistrationStatus,
    /// Credentials and certifications
    #[sea_orm(column_type = "JsonBinary")]
    pub credentials: PartyCredentials,
    /// Additional metadata
    #[sea_orm(column_type = "JsonBinary")]
    pub metadata: serde_json::Value,
    /// Registration submission timestamp
    pub submitted_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Verification timestamp
    pub verified_at: Option<DateTime<Utc>>,
    /// Verification notes
    pub verification_notes: Option<String>,
}

impl Model {
    pub fn new(
        did: String,
        role: PartyRole,
        organization_name: String,
        description: String,
        contact_email: String,
        credentials: PartyCredentials,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            did,
            role,
            organization_name,
            description,
            contact_email,
            status: RegistrationStatus::Pending,
            credentials,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            submitted_at: Utc::now(),
            updated_at: Utc::now(),
            verified_at: None,
            verification_notes: None,
        }
    }

    /// Updates the registration status
    pub fn update_status(&mut self, status: RegistrationStatus, notes: Option<String>) {
        self.status = status;
        self.verification_notes = notes;
        if status == RegistrationStatus::Active {
            self.verified_at = Some(Utc::now());
        }
        self.updated_at = Utc::now();
    }

    /// Updates the credentials
    pub fn update_credentials(&mut self, credentials: PartyCredentials) {
        self.credentials = credentials;
        self.updated_at = Utc::now();
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_credentials() -> PartyCredentials {
        PartyCredentials {
            certifications: vec!["ISO27001".to_string()],
            compliance_standards: vec!["GDPR".to_string()],
            data_ownership_proofs: Some(vec!["proof1".to_string()]),
            infrastructure_capabilities: None,
            processing_capabilities: None,
        }
    }

    #[test]
    fn test_registration_creation() {
        let registration = Model::new(
            "did:example:123".to_string(),
            PartyRole::DataProvider,
            "Test Org".to_string(),
            "Test Description".to_string(),
            "test@example.com".to_string(),
            create_test_credentials(),
        );

        assert_eq!(registration.status, RegistrationStatus::Pending);
        assert!(registration.verified_at.is_none());
    }

    #[test]
    fn test_status_update() {
        let mut registration = Model::new(
            "did:example:123".to_string(),
            PartyRole::DataProvider,
            "Test Org".to_string(),
            "Test Description".to_string(),
            "test@example.com".to_string(),
            create_test_credentials(),
        );

        registration.update_status(
            RegistrationStatus::Active,
            Some("Verification completed".to_string()),
        );

        assert_eq!(registration.status, RegistrationStatus::Active);
        assert!(registration.verified_at.is_some());
        assert_eq!(
            registration.verification_notes,
            Some("Verification completed".to_string())
        );
    }
} 