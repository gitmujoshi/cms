//! Contract Management System - API Module
//! 
//! This module defines the REST API endpoints for the Contract Management System.
//! It provides the HTTP interface for:
//! - Contract management
//! - User operations
//! - Organization management
//! - Authentication
//! - System operations
//!
//! Features:
//! - RESTful endpoints
//! - Request validation
//! - Response formatting
//! - Error handling
//! - Documentation
//!
//! API Categories:
//! - Contract endpoints
//! - User endpoints
//! - Organization endpoints
//! - Authentication endpoints
//! - System endpoints
//!
//! Security Features:
//! - Authentication
//! - Authorization
//! - Input validation
//! - Rate limiting
//! - CORS handling
//!
//! Usage:
//! 1. Configure API routes
//! 2. Handle requests
//! 3. Process responses
//! 4. Manage errors
//! 5. Document endpoints
//!
//! Author: Contract Management System Team
//! License: MIT

mod contracts;

pub use contracts::config as contracts_config; 