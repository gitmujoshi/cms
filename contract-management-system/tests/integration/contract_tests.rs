use actix_web::{test, web, App};
use chrono::Utc;
use uuid::Uuid;

use contract_management_system::{
    api::contracts_config,
    models::{
        Contract, ContractStatus, ContractTerms, ContractType, DataUsageTerms, EncryptionLevel,
        ModelTrainingTerms, Party, PartyRole, SecurityRequirements,
    },
    utils::{db::init_db_pool, init_metrics},
};

async fn setup_test_app() -> actix_web::dev::ServiceResponse {
    // Initialize test database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/contract_management_test".into());
    init_db_pool(&database_url).await.unwrap();
    init_metrics();

    // Create test app
    test::init_service(
        App::new()
            .configure(contracts_config)
            .app_data(web::JsonConfig::default().limit(4096)),
    )
    .await
}

#[actix_web::test]
async fn test_contract_lifecycle() {
    let app = setup_test_app().await;

    // Create contract
    let contract = Contract::new(
        "Test Contract".into(),
        "Integration Test Contract".into(),
        ContractType::DataSharing,
        ContractTerms {
            data_usage_terms: DataUsageTerms {
                allowed_purposes: vec!["Testing".into()],
                usage_restrictions: vec!["Test only".into()],
                retention_period: 30,
                data_handling_requirements: vec!["Secure".into()],
            },
            security_requirements: SecurityRequirements {
                encryption_level: EncryptionLevel::High,
                access_controls: vec!["MFA".into()],
                audit_requirements: vec!["Daily".into()],
            },
            compliance_requirements: vec![],
            model_training_terms: ModelTrainingTerms::default(),
        },
        Utc::now(),
        None,
    );

    // Test contract creation
    let req = test::TestRequest::post()
        .uri("/api/v1/contracts")
        .set_json(&contract)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let created_contract: Contract = test::read_body_json(resp).await;
    assert_eq!(created_contract.title, "Test Contract");
    assert_eq!(created_contract.status, ContractStatus::Draft);

    // Test contract retrieval
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/contracts/{}", created_contract.id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let retrieved_contract: Contract = test::read_body_json(resp).await;
    assert_eq!(retrieved_contract.id, created_contract.id);

    // Test contract update
    let mut updated_contract = created_contract.clone();
    updated_contract.title = "Updated Test Contract".into();
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/contracts/{}", created_contract.id))
        .set_json(&updated_contract)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test contract status update
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/contracts/{}/status",
            created_contract.id
        ))
        .set_json(&ContractStatus::PendingApproval)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test contract list
    let req = test::TestRequest::get()
        .uri("/api/v1/contracts?page=1&per_page=10")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test contract deletion
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/contracts/{}", created_contract.id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_contract_validation() {
    let app = setup_test_app().await;

    // Test invalid contract creation (missing required fields)
    let invalid_contract = serde_json::json!({
        "title": "",
        "description": "Invalid Contract",
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/contracts")
        .set_json(&invalid_contract)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Test invalid contract status transition
    let contract = Contract::new(
        "Test Contract".into(),
        "Test Description".into(),
        ContractType::DataSharing,
        ContractTerms::default(),
        Utc::now(),
        None,
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/contracts")
        .set_json(&contract)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let created_contract: Contract = test::read_body_json(resp).await;

    // Try to activate contract without required signatures
    let req = test::TestRequest::post()
        .uri(&format!(
            "/api/v1/contracts/{}/status",
            created_contract.id
        ))
        .set_json(&ContractStatus::Active)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_contract_metrics() {
    let app = setup_test_app().await;

    // Create a contract and verify metrics
    let contract = Contract::new(
        "Metrics Test Contract".into(),
        "Test Description".into(),
        ContractType::DataSharing,
        ContractTerms::default(),
        Utc::now(),
        None,
    );

    let req = test::TestRequest::post()
        .uri("/api/v1/contracts")
        .set_json(&contract)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify metrics endpoint
    let req = test::TestRequest::get()
        .uri("/metrics")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let metrics = test::read_body(resp).await;
    let metrics_str = String::from_utf8(metrics.to_vec()).unwrap();
    
    // Check if our metrics are present
    assert!(metrics_str.contains("contract_operations_total"));
    assert!(metrics_str.contains("http_request_duration_seconds"));
} 