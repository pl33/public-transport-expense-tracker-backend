use sea_orm_migration::{prelude::*, schema::*};

use crate::m20250323_220823_tag_descriptor::TagDescriptor;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TagEnumOption::Table)
                    .if_not_exists()
                    .col(pk_auto(TagEnumOption::Id))
                    .col(date_time(TagEnumOption::CreatedAt))
                    .col(date_time(TagEnumOption::UpdatedAt))
                    .col(date_time_null(TagEnumOption::DeletedAt))
                    .col(integer(TagEnumOption::TagDescriptorId))
                    .foreign_key(ForeignKey::create()
                                     .name(TagEnumOption::TagDescriptorId.to_string())
                                     .from(TagEnumOption::Table, TagEnumOption::TagDescriptorId)
                                     .to(TagDescriptor::Table, TagDescriptor::Id)
                                     .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(TagEnumOption::Order))
                    .col(string(TagEnumOption::Value))
                    .col(uuid(TagEnumOption::Uuid))
                    .col(string_null(TagEnumOption::Name))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TagEnumOption::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TagEnumOption {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    TagDescriptorId,
    Order,
    Value,
    Uuid,
    Name,
}
