/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "ride_tag")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
    pub ride_id: u32,
    pub tag_descriptor_id: u32,
    pub order: u32,
    pub value_integer: Option<i64>,
    pub value_float: Option<f64>,
    pub value_string: Option<String>,
    pub value_date_time: Option<DateTimeUtc>,
    pub value_enum_option_id: Option<u32>,
    pub remarks: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::ride::Entity",
        from = "Column::RideId",
        to = "super::ride::Column::Id"
    )]
    Ride,
    #[sea_orm(
        belongs_to = "super::tag_descriptor::Entity",
        from = "Column::TagDescriptorId",
        to = "super::tag_descriptor::Column::Id"
    )]
    TagDescriptor,
}

impl Related<super::ride::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ride.def()
    }
}

impl Related<super::tag_descriptor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TagDescriptor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
