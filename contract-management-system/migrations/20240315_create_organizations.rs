use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Organizations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Organizations::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Organizations::Name).string().not_null())
                    .col(ColumnDef::new(Organizations::Description).string())
                    .col(ColumnDef::new(Organizations::Type)
                        .string()
                        .not_null()
                        .default("business"))
                    .col(ColumnDef::new(Organizations::Status)
                        .string()
                        .not_null()
                        .default("active"))
                    .col(ColumnDef::new(Organizations::Website).string())
                    .col(ColumnDef::new(Organizations::Email).string().not_null())
                    .col(ColumnDef::new(Organizations::Phone).string())
                    .col(ColumnDef::new(Organizations::Address).json())
                    .col(
                        ColumnDef::new(Organizations::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Organizations::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for organization name searches
        manager
            .create_index(
                Index::create()
                    .name("idx_organizations_name")
                    .table(Organizations::Table)
                    .col(Organizations::Name)
                    .to_owned(),
            )
            .await?;

        // Index for organization email
        manager
            .create_index(
                Index::create()
                    .name("idx_organizations_email")
                    .table(Organizations::Table)
                    .col(Organizations::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Organizations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Organizations {
    Table,
    Id,
    Name,
    Description,
    Type,
    Status,
    Website,
    Email,
    Phone,
    Address,
    CreatedAt,
    UpdatedAt,
} 