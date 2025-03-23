/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ride")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
    pub user_id: u32,
    pub journey_departure: DateTimeUtc,
    pub journey_arrival: Option<DateTimeUtc>,
    pub location_from: String,
    pub location_to: String,
    pub remarks: Option<String>,
    pub is_template: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::ride_tag::Entity")]
    RideTags,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::ride_tag::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::RideTags.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
