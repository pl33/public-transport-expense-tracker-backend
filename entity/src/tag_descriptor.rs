/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "tag_descriptor")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: u32,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
    pub user_id: u32,
    pub tag_type: TagType,
    pub tag_key: String,
    pub tag_name: Option<String>,
    pub uuid: Uuid,
    pub unit: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)", rename_all = "snake_case")]
pub enum TagType {
    Float,
    Integer,
    String,
    Enum,
    DateTime,
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
    #[sea_orm(has_many = "super::tag_enum_option::Entity")]
    TagEnumOptions,
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

impl Related<super::tag_enum_option::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TagEnumOptions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl TryFrom<String> for TagType {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "integer" => Ok(TagType::Integer),
            "float" => Ok(TagType::Float),
            "string" => Ok(TagType::String),
            "enum" => Ok(TagType::Enum),
            "date_time" => Ok(TagType::DateTime),
            _ => Err("Invalid tag type"),
        }
    }
}

impl Into<String> for TagType {
    fn into(self) -> String {
        match self {
            TagType::Integer => "integer",
            TagType::Float => "float",
            TagType::String => "string",
            TagType::Enum => "enum",
            TagType::DateTime => "date_time",
        }.to_string()
    }
}
