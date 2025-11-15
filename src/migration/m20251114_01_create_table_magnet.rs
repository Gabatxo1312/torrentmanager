use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Magnet::Table)
                    .if_not_exists()
                    .col(pk_auto(Magnet::Id))
                    .col(string(Magnet::TorrentID).unique_key())
                    .col(string(Magnet::Name))
                    .col(string(Magnet::Link))
                    .col(boolean(Magnet::Resolved))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Magnet::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Magnet {
    Table,
    Id,
    TorrentID,
    Name,
    Link,
    Resolved,
}
