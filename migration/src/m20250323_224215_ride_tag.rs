use sea_orm_migration::{prelude::*, schema::*};

use super::m20250323_195423_ride::Ride;
use super::m20250323_220823_tag_descriptor::TagDescriptor;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RideTag::Table)
                    .if_not_exists()
                    .col(pk_auto(RideTag::Id))
                    .col(date_time(RideTag::CreatedAt))
                    .col(date_time(RideTag::UpdatedAt))
                    .col(date_time_null(RideTag::DeletedAt))
                    .col(integer(RideTag::RideId))
                    .foreign_key(ForeignKey::create()
                                     .name(RideTag::RideId.to_string())
                                     .from(RideTag::Table, RideTag::RideId)
                                     .to(Ride::Table, Ride::Id)
                                     .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(RideTag::TagDescriptorId))
                    .foreign_key(ForeignKey::create()
                                     .name(RideTag::TagDescriptorId.to_string())
                                     .from(RideTag::Table, RideTag::TagDescriptorId)
                                     .to(TagDescriptor::Table, TagDescriptor::Id)
                                     .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(RideTag::Order))
                    .col(integer_null(RideTag::ValueInteger))
                    .col(float_null(RideTag::ValueFloat))
                    .col(string_null(RideTag::ValueString))
                    .col(date_time_null(RideTag::ValueDateTime))
                    .col(integer_null(RideTag::ValueEnumOptionId))
                    .col(string_null(RideTag::Remarks))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RideTag::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum RideTag {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    RideId,
    TagDescriptorId,
    Order,
    ValueInteger,
    ValueFloat,
    ValueString,
    ValueDateTime,
    ValueEnumOptionId,
    Remarks,
}
