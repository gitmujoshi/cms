use crate::iam::permissions::{Permission, PermissionSet};
use anyhow::{Context, Result};
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub effect: PolicyEffect,
    pub resources: Vec<ResourcePattern>,
    pub actions: Vec<Permission>,
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PolicyEffect {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePattern {
    pub resource_type: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCondition {
    pub condition_type: ConditionType,
    pub key: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionType {
    StringEquals,
    StringNotEquals,
    StringLike,
    StringNotLike,
    DateEquals,
    DateNotEquals,
    DateLessThan,
    DateGreaterThan,
    IpAddress,
    NotIpAddress,
    BoolEquals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicyRequest {
    pub name: String,
    pub description: Option<String>,
    pub effect: PolicyEffect,
    pub resources: Vec<ResourcePattern>,
    pub actions: Vec<Permission>,
    pub conditions: Vec<PolicyCondition>,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicyRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub effect: Option<PolicyEffect>,
    pub resources: Option<Vec<ResourcePattern>>,
    pub actions: Option<Vec<Permission>>,
    pub conditions: Option<Vec<PolicyCondition>>,
    pub priority: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PolicyFilter {
    pub name_contains: Option<String>,
    pub effect: Option<PolicyEffect>,
    pub resource_type: Option<String>,
    pub has_action: Option<Permission>,
    pub min_priority: Option<i32>,
    pub max_priority: Option<i32>,
}

#[derive(Clone)]
pub struct PolicyService {
    db: Arc<DatabaseConnection>,
}

impl PolicyService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }

    #[instrument(skip(self))]
    pub async fn create_policy(&self, request: CreatePolicyRequest) -> Result<Policy> {
        let now = chrono::Utc::now();
        let policy = Policy {
            id: Uuid::new_v4(),
            name: request.name,
            description: request.description,
            effect: request.effect,
            resources: request.resources,
            actions: request.actions,
            conditions: request.conditions,
            priority: request.priority,
            created_at: now,
            last_modified: now,
        };

        // Store policy in database
        let model = super::models::policy::ActiveModel {
            id: Set(policy.id),
            name: Set(policy.name.clone()),
            description: Set(policy.description.clone()),
            effect: Set(policy.effect.to_string()),
            resources: Set(serde_json::to_value(&policy.resources)?),
            actions: Set(serde_json::to_value(&policy.actions)?),
            conditions: Set(serde_json::to_value(&policy.conditions)?),
            priority: Set(policy.priority),
            created_at: Set(policy.created_at),
            last_modified: Set(policy.last_modified),
        };

        Entity::insert(model)
            .exec(&*self.db)
            .await
            .context("Failed to create policy")?;

        info!("Created new policy: {}", policy.id);
        Ok(policy)
    }

    #[instrument(skip(self))]
    pub async fn get_policy(&self, id: Uuid) -> Result<Option<Policy>> {
        let model = Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .context("Failed to get policy")?;

        Ok(model.map(Into::into))
    }

    #[instrument(skip(self))]
    pub async fn update_policy(&self, id: Uuid, request: UpdatePolicyRequest) -> Result<Policy> {
        let mut model: super::models::policy::ActiveModel = Entity::find_by_id(id)
            .one(&*self.db)
            .await
            .context("Failed to get policy")?
            .ok_or_else(|| anyhow::anyhow!("Policy not found"))?
            .into();

        if let Some(name) = request.name {
            model.name = Set(name);
        }
        if let Some(description) = request.description {
            model.description = Set(Some(description));
        }
        if let Some(effect) = request.effect {
            model.effect = Set(effect.to_string());
        }
        if let Some(resources) = request.resources {
            model.resources = Set(serde_json::to_value(resources)?);
        }
        if let Some(actions) = request.actions {
            model.actions = Set(serde_json::to_value(actions)?);
        }
        if let Some(conditions) = request.conditions {
            model.conditions = Set(serde_json::to_value(conditions)?);
        }
        if let Some(priority) = request.priority {
            model.priority = Set(priority);
        }
        model.last_modified = Set(chrono::Utc::now());

        let updated = model
            .update(&*self.db)
            .await
            .context("Failed to update policy")?;

        info!("Updated policy: {}", id);
        Ok(updated.into())
    }

    #[instrument(skip(self))]
    pub async fn delete_policy(&self, id: Uuid) -> Result<()> {
        Entity::delete_by_id(id)
            .exec(&*self.db)
            .await
            .context("Failed to delete policy")?;

        info!("Deleted policy: {}", id);
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn list_policies(&self, filter: PolicyFilter) -> Result<Vec<Policy>> {
        let mut query = Entity::find();

        if let Some(name_contains) = filter.name_contains {
            query = query.filter(super::models::policy::Column::Name.contains(&name_contains));
        }
        if let Some(effect) = filter.effect {
            query = query.filter(super::models::policy::Column::Effect.eq(effect.to_string()));
        }
        if let Some(min_priority) = filter.min_priority {
            query = query.filter(super::models::policy::Column::Priority.gte(min_priority));
        }
        if let Some(max_priority) = filter.max_priority {
            query = query.filter(super::models::policy::Column::Priority.lte(max_priority));
        }

        let models = query
            .all(&*self.db)
            .await
            .context("Failed to list policies")?;

        let mut policies: Vec<Policy> = models.into_iter().map(Into::into).collect();

        // Additional filtering that can't be done at the database level
        if let Some(resource_type) = filter.resource_type {
            policies.retain(|p| p.resources.iter().any(|r| r.resource_type == resource_type));
        }
        if let Some(action) = filter.has_action {
            policies.retain(|p| p.actions.contains(&action));
        }

        Ok(policies)
    }

    #[instrument(skip(self))]
    pub async fn evaluate_access(
        &self,
        identity_id: Uuid,
        resource: &ResourcePattern,
        action: Permission,
        context: &PolicyEvaluationContext,
    ) -> Result<bool> {
        // Get all policies that could affect this access
        let policies = self
            .list_policies(PolicyFilter {
                effect: None,
                resource_type: Some(resource.resource_type.clone()),
                has_action: Some(action.clone()),
                ..Default::default()
            })
            .await?;

        // Sort policies by priority (highest first)
        let mut policies = policies;
        policies.sort_by_key(|p| -p.priority);

        // Evaluate policies in order
        for policy in policies {
            if self.policy_matches(&policy, resource, &action, context)? {
                return Ok(policy.effect == PolicyEffect::Allow);
            }
        }

        // If no policy matches, deny by default
        Ok(false)
    }

    fn policy_matches(
        &self,
        policy: &Policy,
        resource: &ResourcePattern,
        action: &Permission,
        context: &PolicyEvaluationContext,
    ) -> Result<bool> {
        // Check if action is covered by policy
        if !policy.actions.contains(action) {
            return Ok(false);
        }

        // Check if resource matches any resource pattern in policy
        let resource_matches = policy.resources.iter().any(|pattern| {
            if pattern.resource_type != resource.resource_type {
                return false;
            }
            // Simple wildcard matching for now
            if pattern.pattern == "*" {
                return true;
            }
            pattern.pattern == resource.pattern
        });

        if !resource_matches {
            return Ok(false);
        }

        // Evaluate all conditions
        for condition in &policy.conditions {
            if !self.evaluate_condition(condition, context)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn evaluate_condition(
        &self,
        condition: &PolicyCondition,
        context: &PolicyEvaluationContext,
    ) -> Result<bool> {
        let context_value = context.get(&condition.key).ok_or_else(|| {
            anyhow::anyhow!("Missing context value for condition key: {}", condition.key)
        })?;

        match condition.condition_type {
            ConditionType::StringEquals => Ok(condition.values.contains(context_value)),
            ConditionType::StringNotEquals => Ok(!condition.values.contains(context_value)),
            ConditionType::StringLike => Ok(condition
                .values
                .iter()
                .any(|pattern| wildcard_match(pattern, context_value))),
            ConditionType::StringNotLike => Ok(!condition
                .values
                .iter()
                .any(|pattern| wildcard_match(pattern, context_value))),
            ConditionType::DateEquals => {
                let context_date = parse_date(context_value)?;
                Ok(condition
                    .values
                    .iter()
                    .any(|v| parse_date(v).map(|d| d == context_date).unwrap_or(false)))
            }
            ConditionType::DateNotEquals => {
                let context_date = parse_date(context_value)?;
                Ok(!condition
                    .values
                    .iter()
                    .any(|v| parse_date(v).map(|d| d == context_date).unwrap_or(false)))
            }
            ConditionType::DateLessThan => {
                let context_date = parse_date(context_value)?;
                Ok(condition.values.iter().any(|v| {
                    parse_date(v)
                        .map(|d| context_date < d)
                        .unwrap_or(false)
                }))
            }
            ConditionType::DateGreaterThan => {
                let context_date = parse_date(context_value)?;
                Ok(condition.values.iter().any(|v| {
                    parse_date(v)
                        .map(|d| context_date > d)
                        .unwrap_or(false)
                }))
            }
            ConditionType::IpAddress => Ok(condition.values.contains(context_value)),
            ConditionType::NotIpAddress => Ok(!condition.values.contains(context_value)),
            ConditionType::BoolEquals => {
                let context_bool = context_value.parse::<bool>()?;
                Ok(condition
                    .values
                    .iter()
                    .any(|v| v.parse::<bool>().map(|b| b == context_bool).unwrap_or(false)))
            }
        }
    }
}

pub type PolicyEvaluationContext = std::collections::HashMap<String, String>;

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = regex::escape(pattern).replace("\\*", ".*");
    regex::Regex::new(&format!("^{}$", pattern))
        .map(|re| re.is_match(value))
        .unwrap_or(false)
}

fn parse_date(value: &str) -> Result<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .context("Invalid date format")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wildcard_match() {
        assert!(wildcard_match("test*", "test123"));
        assert!(wildcard_match("*test", "mytest"));
        assert!(wildcard_match("*test*", "mytestvalue"));
        assert!(!wildcard_match("test", "test123"));
        assert!(!wildcard_match("*test", "testing"));
    }

    #[test]
    fn test_parse_date() {
        let date = parse_date("2024-03-20T12:00:00Z").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 3);
        assert_eq!(date.day(), 20);
 