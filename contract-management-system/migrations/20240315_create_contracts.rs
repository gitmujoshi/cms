use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Contracts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Contracts::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Contracts::Title).string().not_null())
                    .col(ColumnDef::new(Contracts::Description).text().not_null())
                    .col(ColumnDef::new(Contracts::ContractType).string().not_null())
                    .col(ColumnDef::new(Contracts::ProviderId).uuid().not_null())
                    .col(ColumnDef::new(Contracts::ConsumerId).uuid().not_null())
                    .col(ColumnDef::new(Contracts::Terms).json().not_null())
                    .col(ColumnDef::new(Contracts::Status).string().not_null())
                    .col(ColumnDef::new(Contracts::ValidFrom).timestamp().not_null())
                    .col(ColumnDef::new(Contracts::ValidUntil).timestamp())
                    .col(
                        ColumnDef::new(Contracts::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Contracts::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contracts_provider")
                    .table(Contracts::Table)
                    .col(Contracts::ProviderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contracts_consumer")
                    .table(Contracts::Table)
                    .col(Contracts::ConsumerId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contracts_status")
                    .table(Contracts::Table)
                    .col(Contracts::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Contracts::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Contracts {
    Table,
    Id,
    Title,
    Description,
    ContractType,
    ProviderId,
    ConsumerId,
    Terms,
    Status,
    ValidFrom,
    ValidUntil,
    CreatedAt,
    UpdatedAt,
} 