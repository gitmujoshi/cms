use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum Permission {
    // Contract Management
    CreateContract,
    ReadContract,
    UpdateContract,
    DeleteContract,
    ListContracts,
    SignContract,
    ValidateContract,

    // Identity Management
    CreateIdentity,
    ReadIdentity,
    UpdateIdentity,
    DeleteIdentity,
    ListIdentities,
    ManageCredentials,

    // Role Management
    CreateRole,
    ReadRole,
    UpdateRole,
    DeleteRole,
    ListRoles,
    AssignRole,

    // Model Training
    CreateTrainingJob,
    ReadTrainingJob,
    UpdateTrainingJob,
    DeleteTrainingJob,
    ListTrainingJobs,
    MonitorTraining,

    // Data Access
    ReadTrainingData,
    WriteTrainingData,
    DeleteTrainingData,
    ShareTrainingData,

    // Enclave Operations
    CreateEnclave,
    ReadEnclave,
    UpdateEnclave,
    DeleteEnclave,
    ListEnclaves,
    AttestEnclave,

    // Audit Operations
    ReadAuditLogs,
    ExportAuditLogs,
    ConfigureAuditing,

    // System Administration
    ConfigureSystem,
    ManageEncryption,
    ViewMetrics,
    ManageBackups,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PermissionSet {
    permissions: std::collections::HashSet<Permission>,
}

impl PermissionSet {
    pub fn new() -> Self {
        Self {
            permissions: std::collections::HashSet::new(),
        }
    }

    pub fn add(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    pub fn remove(&mut self, permission: &Permission) {
        self.permissions.remove(permission);
    }

    pub fn has(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_any(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.has(p))
    }

    pub fn has_all(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.has(p))
    }

    pub fn merge(&mut self, other: &PermissionSet) {
        self.permissions.extend(other.permissions.iter().cloned());
    }

    pub fn intersect(&mut self, other: &PermissionSet) {
        self.permissions.retain(|p| other.permissions.contains(p));
    }

    pub fn clear(&mut self) {
        self.permissions.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Permission> {
        self.permissions.iter()
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<Permission> for PermissionSet {
    fn from_iter<I: IntoIterator<Item = Permission>>(iter: I) -> Self {
        let mut set = PermissionSet::new();
        for permission in iter {
            set.add(permission);
        }
        set
    }
}

// Predefined permission sets for common roles
pub fn get_admin_permissions() -> PermissionSet {
    use Permission::*;
    [
        CreateContract, ReadContract, UpdateContract, DeleteContract, ListContracts,
        SignContract, ValidateContract, CreateIdentity, ReadIdentity, UpdateIdentity,
        DeleteIdentity, ListIdentities, ManageCredentials, CreateRole, ReadRole,
        UpdateRole, DeleteRole, ListRoles, AssignRole, CreateTrainingJob,
        ReadTrainingJob, UpdateTrainingJob, DeleteTrainingJob, ListTrainingJobs,
        MonitorTraining, ReadTrainingData, WriteTrainingData, DeleteTrainingData,
        ShareTrainingData, CreateEnclave, ReadEnclave, UpdateEnclave, DeleteEnclave,
        ListEnclaves, AttestEnclave, ReadAuditLogs, ExportAuditLogs,
        ConfigureAuditing, ConfigureSystem, ManageEncryption, ViewMetrics,
        ManageBackups,
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn get_model_trainer_permissions() -> PermissionSet {
    use Permission::*;
    [
        ReadContract, ListContracts, CreateTrainingJob, ReadTrainingJob,
        UpdateTrainingJob, ListTrainingJobs, MonitorTraining, ReadTrainingData,
        WriteTrainingData, ReadEnclave, ListEnclaves, ViewMetrics,
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn get_data_provider_permissions() -> PermissionSet {
    use Permission::*;
    [
        ReadContract, ListContracts, SignContract, ReadTrainingJob,
        ListTrainingJobs, MonitorTraining, ReadTrainingData, WriteTrainingData,
        ShareTrainingData, ReadEnclave, ListEnclaves, ViewMetrics,
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn get_auditor_permissions() -> PermissionSet {
    use Permission::*;
    [
        ReadContract, ListContracts, ReadTrainingJob, ListTrainingJobs,
        MonitorTraining, ReadTrainingData, ReadEnclave, ListEnclaves,
        ReadAuditLogs, ExportAuditLogs, ViewMetrics,
    ]
    .iter()
    .cloned()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_operations() {
        let mut set = PermissionSet::new();
        assert!(set.is_empty());

        set.add(Permission::CreateContract);
        set.add(Permission::ReadContract);
        assert_eq!(set.len(), 2);
        assert!(set.has(&Permission::CreateContract));
        assert!(set.has(&Permission::ReadContract));
        assert!(!set.has(&Permission::DeleteContract));

        set.remove(&Permission::CreateContract);
        assert_eq!(set.len(), 1);
        assert!(!set.has(&Permission::CreateContract));
        assert!(set.has(&Permission::ReadContract));

        let mut other_set = PermissionSet::new();
        other_set.add(Permission::ReadContract);
        other_set.add(Permission::UpdateContract);

        set.merge(&other_set);
        assert_eq!(set.len(), 2);
        assert!(set.has(&Permission::ReadContract));
        assert!(set.has(&Permission::UpdateContract));
    }

    #[test]
    fn test_predefined_permission_sets() {
        let admin = get_admin_permissions();
        let trainer = get_model_trainer_permissions();
        let provider = get_data_provider_permissions();
        let auditor = get_auditor_permissions();

        // Admin should have all permissions
        assert!(admin.has(&Permission::ConfigureSystem));
        assert!(admin.has(&Permission::ManageEncryption));

        // Model trainer should have training-related permissions
        assert!(trainer.has(&Permission::CreateTrainingJob));
        assert!(trainer.has(&Permission::MonitorTraining));
        assert!(!trainer.has(&Permission::ConfigureSystem));

        // Data provider should have data-related permissions
        assert!(provider.has(&Permission::ShareTrainingData));
        assert!(provider.has(&Permission::WriteTrainingData));
        assert!(!provider.has(&Permission::ConfigureSystem));

        // Auditor should have read-only and audit permissions
        assert!(auditor.has(&Permission::ReadAuditLogs));
        assert!(auditor.has(&Permission::ExportAuditLogs));
        assert!(!auditor.has(&Permission::ConfigureSystem));
    }
} 