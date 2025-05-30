//! Contract Management System - Repository Model
//! 
//! This module defines the data repository model for the Contract Management System.
//! It provides the database access layer and data structures for:
//! - Data persistence
//! - Query operations
//! - Transaction management
//! - Data relationships
//! - Data validation
//!
//! Features:
//! - Repository pattern implementation
//! - Query building
//! - Transaction handling
//! - Relationship management
//! - Data validation
//!
//! Data Operations:
//! - CRUD operations
//! - Complex queries
//! - Batch operations
//! - Relationship queries
//! - Data aggregation
//!
//! Integration Points:
//! - Database connection
//! - Entity models
//! - Service layer
//! - Transaction system
//! - Cache system
//!
//! Usage:
//! 1. Initialize repository
//! 2. Perform data operations
//! 3. Handle transactions
//! 4. Manage relationships
//! 5. Validate data
//!
//! Author: Contract Management System Team
//! License: MIT

use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use uuid::Uuid;

use crate::models::contract::{Contract, ContractStatus};
use crate::utils::error::{AppError, AppResult};

pub struct ContractRepository {
    db: DatabaseConnection,
}

impl ContractRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn create(&self, contract: Contract) -> AppResult<Contract> {
        let active_model = contract.into_active_model();
        let result = active_model.insert(&self.db).await?;
        Ok(result.into())
    }

    pub async fn get_by_id(&self, id: Uuid) -> AppResult<Contract> {
        let contract = Contract::find_by_id(id)
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Contract not found: {}", id)))?;
        Ok(contract)
    }

    pub async fn list(
        &self,
        page: u64,
        per_page: u64,
        status: Option<ContractStatus>,
    ) -> AppResult<(Vec<Contract>, u64)> {
        let mut query = Contract::find();

        if let Some(status) = status {
            query = query.filter(Contract::Column::Status.eq(status));
        }

        let paginator = query
            .order_by_desc(Contract::Column::CreatedAt)
            .paginate(&self.db, per_page);

        let total = paginator.num_items().await?;
        let contracts = paginator.fetch_page(page - 1).await?;

        Ok((contracts, total))
    }

    pub async fn update(&self, id: Uuid, contract: Contract) -> AppResult<Contract> {
        let existing = self.get_by_id(id).await?;
        let mut active_model = existing.into_active_model();

        // Update fields
        active_model.title = Set(contract.title);
        active_model.description = Set(contract.description);
        active_model.status = Set(contract.status);
        active_model.terms = Set(contract.terms);
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(&self.db).await?;
        Ok(updated.into())
    }

    pub async fn delete(&self, id: Uuid) -> AppResult<()> {
        let result = Contract::delete_by_id(id).exec(&self.db).await?;
        if result.rows_affected == 0 {
            return Err(AppError::NotFound(format!("Contract not found: {}", id)));
        }
        Ok(())
    }

    pub async fn update_status(&self, id: Uuid, status: ContractStatus) -> AppResult<Contract> {
        let existing = self.get_by_id(id).await?;
        let mut active_model = existing.into_active_model();

        active_model.status = Set(status);
        active_model.updated_at = Set(chrono::Utc::now());

        let updated = active_model.update(&self.db).await?;
        Ok(updated.into())
    }

    pub async fn get_active_contracts(&self) -> AppResult<Vec<Contract>> {
        let contracts = Contract::find()
            .filter(Contract::Column::Status.eq(ContractStatus::Active))
            .order_by_desc(Contract::Column::CreatedAt)
            .all(&self.db)
            .await?;
        Ok(contracts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::contract::{ContractType, Party};
    use crate::utils::db::init_db_pool;

    async fn setup() -> ContractRepository {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/contract_management".into());
        
        init_db_pool(&database_url).await.unwrap();
        let db = crate::utils::db::get_db_pool().await;
        ContractRepository::new((*db).clone())
    }

    #[tokio::test]
    async fn test_contract_crud() {
        let repo = setup().await;

        // Create
        let contract = Contract::new(
            "Test Contract".into(),
            "Test Description".into(),
            ContractType::DataSharing,
            vec![],
            Default::default(),
            chrono::Utc::now(),
            None,
        );

        let created = repo.create(contract.clone()).await.unwrap();
        assert_eq!(created.title, "Test Contract");

        // Read
        let found = repo.get_by_id(created.id).await.unwrap();
        assert_eq!(found.id, created.id);

        // Update
        let mut updated = created.clone();
        updated.title = "Updated Title".into();
        let updated = repo.update(created.id, updated).await.unwrap();
        assert_eq!(updated.title, "Updated Title");

        // Delete
        repo.delete(created.id).await.unwrap();
        assert!(repo.get_by_id(created.id).await.is_err());
    }
} 