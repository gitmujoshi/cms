use crate::iam::permissions::Permission;
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permissions: Option<Vec<Permission>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RoleFilter {
    pub name_contains: Option<String>,
    pub has_permission: Option<Permission>,
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    pub created_before: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone)]
pub struct RoleService {
    db: Arc<DatabaseConnection>,
}

impl RoleService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    #[instrument(skip(self))]
    pub async fn create_role(&self, request: CreateRoleRequest) -> Result<Role> {
        let now = chrono::Utc::now();
        let role = Role {
            id: Uuid::new_v4(),
            name: request.name,
            description: request.description,
            permissions: request.permissions,
            created_at: now,
            last_modified: now,
        };

        // Store role in database
        let model = super::models::role::ActiveModel {
            id: Set(role.id),
            name: Set(role.name.clone()),
            description: Set(role.description.clone()),
            permissions: Set(serde_json::to_value(&role.permissions)?),
            created_at: Set(role.created_at),
            last_modified: Set(role.last_modified),
        };

        Entity::insert(model)
            .exec(&*self.db)
            .await
            .context("Failed to create role")?;

        info!("Created new role: {}", role.id);
        Ok(role)
    }

    #[instrument(skip(self))]
    pub async fn get_role(&self, id: Uuid) -> Result<Option<Role>> {
        let model = Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .context("Failed to get role")?;

        Ok(model.map(Into::into))
    }

    #[instrument(skip(self))]
    pub async fn update_role(&self, id: Uuid, request: UpdateRoleRequest) -> Result<Role> {
        let mut model: super::models::role::ActiveModel = Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .context("Failed to get role")?
            .ok_or_else(|| anyhow::anyhow!("Role not found"))?
            .into();

        if let Some(name) = request.name {
            model.name = Set(name);
        }
        if let Some(description) = request.description {
            model.description = Set(Some(description));
        }
        if let Some(permissions) = request.permissions {
            model.permissions = Set(serde_json::to_value(permissions)?);
        }
        model.last_modified = Set(chrono::Utc::now());

        let updated = model
            .update(&*self.db)
            .await
            .context("Failed to update role")?;

        info!("Updated role: {}", id);
        Ok(updated.into())
    }

    #[instrument(skip(self))]
    pub async fn delete_role(&self, id: Uuid) -> Result<()> {
        Entity::delete_by_id(id)
            .exec(&*self.db)
            .await
            .context("Failed to delete role")?;

        info!("Deleted role: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn list_roles(&self, filter: RoleFilter) -> Result<Vec<Role>> {
        let mut query = Entity::find();

        if let Some(name_contains) = filter.name_contains {
            query = query.filter(super::models::role::Column::Name.contains(&name_contains));
        }
        if let Some(created_after) = filter.created_after {
            query = query.filter(super::models::role::Column::CreatedAt.gte(created_after));
        }
        if let Some(created_before) = filter.created_before {
            query = query.filter(super::models::role::Column::CreatedAt.lte(created_before));
        }

        let models = query
            .all(&*self.db)
            .await
            .context("Failed to list roles")?;

        let mut roles: Vec<Role> = models.into_iter().map(Into::into).collect();

        // Filter by permission if specified
        if let Some(permission) = filter.has_permission {
            roles.retain(|role| role.permissions.contains(&permission));
        }

        Ok(roles)
    }

    #[instrument(skip(self))]
    pub async fn assign_role_to_identity(&self, role_id: Uuid, identity_id: Uuid) -> Result<()> {
        let model = super::models::role_assignment::ActiveModel {
            role_id: Set(role_id),
            identity_id: Set(identity_id),
            assigned_at: Set(chrono::Utc::now()),
        };

        Entity::insert(model)
            .exec(&*self.db)
            .await
            .context("Failed to assign role to identity")?;

        info!("Assigned role {} to identity {}", role_id, identity_id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn remove_role_from_identity(&self, role_id: Uuid, identity_id: Uuid) -> Result<()> {
        Entity::delete_many()
            .filter(super::models::role_assignment::Column::RoleId.eq(role_id))
            .filter(super::models::role_assignment::Column::IdentityId.eq(identity_id))
            .exec(&*self.db)
            .await
            .context("Failed to remove role from identity")?;

        info!("Removed role {} from identity {}", role_id, identity_id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_identity_roles(&self, identity_id: Uuid) -> Result<Vec<Role>> {
        let assignments = Entity::find()
            .filter(super::models::role_assignment::Column::IdentityId.eq(identity_id))
            .all(&*self.db)
            .await
            .context("Failed to get identity role assignments")?;

        let role_ids: Vec<Uuid> = assignments
            .into_iter()
            .map(|a| a.role_id)
            .collect();

        let roles = Entity::find()
            .filter(super::models::role::Column::Id.is_in(role_ids))
            .all(&*self.db)
            .await
            .context("Failed to get roles")?;

        Ok(roles.into_iter().map(Into::into).collect())
    }

    #[instrument(skip(self))]
    pub async fn check_identity_has_permission(
        &self,
        identity_id: Uuid,
        permission: Permission,
    ) -> Result<bool> {
        let roles = self.get_identity_roles(identity_id).await?;
        Ok(roles.iter().any(|role| role.permissions.contains(&permission)))
    }
} 