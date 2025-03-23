/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tag_enum_option")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
    pub tag_descriptor_id: u32,
    pub order: u32,
    pub value: String,
    pub uuid: Uuid,
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tag_descriptor::Entity",
        from = "Column::TagDescriptorId",
        to = "super::tag_descriptor::Column::Id"
    )]
    TagDescriptor,
}

impl Related<super::tag_descriptor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TagDescriptor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
