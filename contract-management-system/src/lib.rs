pub mod api;
pub mod models;
pub mod services;
pub mod utils;

// Contract Creation Module
pub mod contracts {
    pub mod templates;
    pub mod permissions;
    pub mod signatures;
}

// Nitro Enclave Integration
pub mod enclave {
    pub mod attestation;
    pub mod verification;
    pub mod compute;
}

// Dataset Access Control
pub mod access {
    pub mod tokens;
    pub mod permissions;
    pub mod time_bounds;
}

// Audit & Compliance
pub mod audit {
    pub mod logging;
    pub mod verification;
    pub mod reporting;
}

// Model Training Orchestration
pub mod training {
    pub mod pipeline;
    pub mod validation;
    pub mod artifacts;
}

// Security Components
pub mod security {
    pub mod encryption;
    pub mod key_management;
    pub mod zero_knowledge;
} 