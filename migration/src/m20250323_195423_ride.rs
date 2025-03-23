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
                    .table(Ride::Table)
                    .if_not_exists()
                    .col(pk_auto(Ride::Id))
                    .col(date_time(Ride::CreatedAt))
                    .col(date_time(Ride::UpdatedAt))
                    .col(date_time_null(Ride::DeletedAt))
                    .col(integer(Ride::UserId))
                    .foreign_key(ForeignKey::create()
                        .name(Ride::UserId.to_string())
                        .from(Ride::Table, Ride::UserId)
                        .to(User::Table, User::Id)
                        .on_delete(ForeignKeyAction::Restrict),
                    )
                    .col(date_time(Ride::JourneyDeparture))
                    .col(date_time_null(Ride::JourneyArrival))
                    .col(string(Ride::LocationFrom))
                    .col(string(Ride::LocationTo))
                    .col(string_null(Ride::Remarks))
                    .col(boolean(Ride::IsTemplate))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Ride::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Ride {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    UserId,
    JourneyDeparture,
    JourneyArrival,
    LocationFrom,
    LocationTo,
    Remarks,
    IsTemplate,
}
