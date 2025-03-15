use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ContractSignatures::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ContractSignatures::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ContractSignatures::ContractId).uuid().not_null())
                    .col(ColumnDef::new(ContractSignatures::SignerId).uuid().not_null())
                    .col(ColumnDef::new(ContractSignatures::SignatureType).string().not_null())
                    .col(ColumnDef::new(ContractSignatures::Signature).text().not_null())
                    .col(
                        ColumnDef::new(ContractSignatures::SignedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_contract_signatures_contract")
                            .from(ContractSignatures::Table, ContractSignatures::ContractId)
                            .to(Contracts::Table, Contracts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contract_signatures_contract")
                    .table(ContractSignatures::Table)
                    .col(ContractSignatures::ContractId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_contract_signatures_signer")
                    .table(ContractSignatures::Table)
                    .col(ContractSignatures::SignerId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ContractSignatures::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ContractSignatures {
    Table,
    Id,
    ContractId,
    SignerId,
    SignatureType,
    Signature,
    SignedAt,
}

#[derive(DeriveIden)]
enum Contracts {
    Table,
    Id,
} 