//! Contract Management System - Core Library
//! 
//! This file defines the core library structure and module organization for the Contract Management System.
//! It serves as the central module registry and documentation hub for the entire system.
//!
//! The library is organized into several key modules:
//! - api: REST API endpoints and handlers
//! - models: Data models and database schemas
//! - services: Business logic implementation
//! - utils: Utility functions and helpers
//!
//! Specialized modules include:
//! - contracts: Contract management and processing
//! - enclave: Secure computation using AWS Nitro Enclaves
//! - access: Dataset access control
//! - audit: Audit logging and compliance
//! - training: Model training orchestration
//! - security: Core security features
//!
//! Usage:
//! This library is used as the foundation for the Contract Management System.
//! It provides all the core functionality needed by the main application.
//!
//! Author: Contract Management System Team
//! License: MIT

pub mod api;
pub mod models;
pub mod services;
pub mod utils;

/// Contract Management Module
/// 
/// Handles all contract-related functionality including:
/// - Contract template management
/// - Permission controls
/// - Digital signature processing
pub mod contracts {
    pub mod templates;    // Contract template definitions and management
    pub mod permissions;  // Access control for contract operations
    pub mod signatures;   // Digital signature handling and verification
}

/// Nitro Enclave Integration Module
/// 
/// Provides secure computation capabilities using AWS Nitro Enclaves:
/// - Attestation for enclave verification
/// - Secure computation execution
/// - Result verification
pub mod enclave {
    pub mod attestation;   // Enclave attestation and verification
    pub mod verification;  // Result verification and validation
    pub mod compute;       // Secure computation execution
}

/// Dataset Access Control Module
/// 
/// Manages access to sensitive datasets with:
/// - Token-based access control
/// - Fine-grained permissions
/// - Time-bound access restrictions
pub mod access {
    pub mod tokens;       // Access token generation and validation
    pub mod permissions;  // Permission management and enforcement
    pub mod time_bounds;  // Time-based access restrictions
}

/// Audit & Compliance Module
/// 
/// Handles system auditing and compliance requirements:
/// - Comprehensive logging
/// - Verification of operations
/// - Compliance reporting
pub mod audit {
    pub mod logging;     // Audit logging and event tracking
    pub mod verification;// Operation verification and validation
    pub mod reporting;   // Compliance report generation
}

/// Model Training Orchestration Module
/// 
/// Manages the machine learning model training process:
/// - Training pipeline management
/// - Model validation
/// - Artifact management
pub mod training {
    pub mod pipeline;    // Training pipeline orchestration
    pub mod validation;  // Model validation and testing
    pub mod artifacts;   // Model artifact management
}

/// Security Components Module
/// 
/// Core security features including:
/// - Encryption/decryption operations
/// - Key management
/// - Zero-knowledge proof implementations
pub mod security {
    pub mod encryption;     // Encryption and decryption operations
    pub mod key_management; // Cryptographic key management
    pub mod zero_knowledge; // Zero-knowledge proof implementations
} 