use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuditLogs::UserId).uuid().not_null())
                    .col(ColumnDef::new(AuditLogs::ResourceType).string().not_null())
                    .col(ColumnDef::new(AuditLogs::ResourceId).uuid())
                    .col(ColumnDef::new(AuditLogs::Action).string().not_null())
                    .col(ColumnDef::new(AuditLogs::Details).json().not_null())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string())
                    .col(ColumnDef::new(AuditLogs::UserAgent).string())
                    .col(
                        ColumnDef::new(AuditLogs::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for efficient querying by resource
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_resource")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::ResourceType)
                    .col(AuditLogs::ResourceId)
                    .to_owned(),
            )
            .await?;

        // Index for user activity queries
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_user")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::UserId)
                    .to_owned(),
            )
            .await?;

        // Index for timestamp-based queries
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_created_at")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    UserId,
    ResourceType,
    ResourceId,
    Action,
    Details,
    IpAddress,
    UserAgent,
    CreatedAt,
} 