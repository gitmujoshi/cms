use lazy_static::lazy_static;
use prometheus::{
    register_histogram_vec, register_int_counter_vec, HistogramVec, IntCounterVec,
};

lazy_static! {
    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds",
        &["method", "endpoint", "status"]
    )
    .unwrap();

    pub static ref CONTRACT_OPERATIONS: IntCounterVec = register_int_counter_vec!(
        "contract_operations_total",
        "Total number of contract operations",
        &["operation", "status"]
    )
    .unwrap();

    pub static ref ACTIVE_CONTRACTS: IntCounterVec = register_int_counter_vec!(
        "active_contracts_total",
        "Total number of active contracts",
        &["type"]
    )
    .unwrap();

    pub static ref ENCLAVE_OPERATIONS: IntCounterVec = register_int_counter_vec!(
        "enclave_operations_total",
        "Total number of enclave operations",
        &["operation", "status"]
    )
    .unwrap();

    pub static ref DATABASE_QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "database_query_duration_seconds",
        "Database query duration in seconds",
        &["query_type"]
    )
    .unwrap();
}

pub fn init_metrics() {
    // Initialize prometheus metrics endpoint
    use actix_web_prom::PrometheusMetricsBuilder;
    use std::collections::HashSet;

    let mut labels = HashSet::new();
    labels.insert("service".to_string());
    
    PrometheusMetricsBuilder::new("contract_management")
        .endpoint("/metrics")
        .const_labels(labels)
        .build()
        .unwrap();
}

pub async fn record_contract_operation(operation: &str, status: &str) {
    CONTRACT_OPERATIONS
        .with_label_values(&[operation, status])
        .inc();
}

pub async fn record_enclave_operation(operation: &str, status: &str) {
    ENCLAVE_OPERATIONS
        .with_label_values(&[operation, status])
        .inc();
}

pub async fn update_active_contracts_metric(contract_type: &str, count: i64) {
    ACTIVE_CONTRACTS
        .with_label_values(&[contract_type])
        .inc_by(count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_recording() {
        // Record a contract operation
        record_contract_operation("create", "success").await;
        
        // Record an enclave operation
        record_enclave_operation("attestation", "success").await;
        
        // Update active contracts
        update_active_contracts_metric("data_sharing", 1).await;

        // Verify metrics were recorded
        let contract_ops = CONTRACT_OPERATIONS
            .with_label_values(&["create", "success"])
            .get();
        assert_eq!(contract_ops, 1);

        let enclave_ops = ENCLAVE_OPERATIONS
            .with_label_values(&["attestation", "success"])
            .get();
        assert_eq!(enclave_ops, 1);

        let active_contracts = ACTIVE_CONTRACTS
            .with_label_values(&["data_sharing"])
            .get();
        assert_eq!(active_contracts, 1);
    }
} 