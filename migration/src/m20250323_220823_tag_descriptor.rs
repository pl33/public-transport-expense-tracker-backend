use sea_orm_migration::{prelude::*, schema::*};

use super::m20250316_204923_user::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TagDescriptor::Table)
                    .if_not_exists()
                    .col(pk_auto(TagDescriptor::Id))
                    .col(date_time(TagDescriptor::CreatedAt))
                    .col(date_time(TagDescriptor::UpdatedAt))
                    .col(date_time_null(TagDescriptor::DeletedAt))
                    .col(integer(TagDescriptor::UserId))
                    .foreign_key(ForeignKey::create()
                                     .name(TagDescriptor::UserId.to_string())
                                     .from(TagDescriptor::Table, TagDescriptor::UserId)
                                     .to(User::Table, User::Id)
                                     .on_delete(ForeignKeyAction::Restrict),
                    )
                    .col(string(TagDescriptor::TagType))
                    .col(string(TagDescriptor::TagKey))
                    .col(string_null(TagDescriptor::TagName))
                    .col(uuid(TagDescriptor::Uuid))
                    .col(string_null(TagDescriptor::Unit))
                    .col(string_null(TagDescriptor::Remarks))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TagDescriptor::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum TagDescriptor {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    UserId,
    TagType,
    TagKey,
    TagName,
    Uuid,
    Unit,
    Remarks,
}
