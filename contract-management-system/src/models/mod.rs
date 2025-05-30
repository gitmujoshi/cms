//! Contract Management System - Data Models Module
//! 
//! This module defines the core data models for the Contract Management System.
//! It provides the database schema and data structures for:
//! - Users and authentication
//! - Organizations and departments
//! - Contracts and templates
//! - Signatures and parties
//! - Audit logs and events
//!
//! Features:
//! - Database schema definitions
//! - Data validation
//! - Relationship management
//! - Type safety
//! - Serialization support
//!
//! Model Categories:
//! - User models
//! - Organization models
//! - Contract models
//! - Signature models
//! - Audit models
//!
//! Usage:
//! 1. Import required models
//! 2. Use models for data operations
//! 3. Handle relationships
//! 4. Perform validation
//!
//! Author: Contract Management System Team
//! License: MIT

mod contract;
mod party;
mod repository;
mod signature;

pub use contract::{
    ComplianceRequirement, Contact, ContactInfo, ContractStatus, ContractTerms, ContractType,
    DataUsageTerms, EncryptionLevel, ModelTrainingTerms, ResourceLimits, SecurityRequirements,
    Model as Contract,
};
pub use party::{Model as Party, PartyRole};
pub use repository::ContractRepository;
pub use signature::{Model as Signature, VerificationMethod}; 