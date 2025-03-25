/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use serde::{Deserialize, Serialize};
use rocket_okapi::okapi::schemars;
use sea_orm::{
    prelude::*,
    Set,
    NotSet,
};
use rand;
use uuid;
use entity::tag_descriptor;
use entity::tag_enum_option;
use super::error::CurdError;

/// JSON structure
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct TagOption {
    #[serde(skip_deserializing)]
    id: u32,
    #[serde(skip_deserializing)]
    tag_id: u32,
    pub order: u32,
    pub value: String,
    #[serde(skip_deserializing)]
    uuid: String,
    pub name: Option<String>,
    #[serde(skip_deserializing)]
    display_name: String,
}

impl From<tag_enum_option::Model> for TagOption {
    fn from(model: tag_enum_option::Model) -> Self {
        Self {
            id: model.id,
            tag_id: model.tag_descriptor_id,
            order: model.order,
            display_name: match &model.name {
                Some(value) => value.clone(),
                None => model.value.clone(),
            },
            value: model.value,
            uuid: model.uuid.to_string(),
            name: model.name,
        }
    }
}

impl TagOption {
    /// Getter for [id]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Getter for [tag_id]
    pub fn tag_id(&self) -> u32 {
        self.tag_id
    }

    /// Getter for [uuid]
    pub fn uuid(&self) -> &String {
        &self.uuid
    }

    /// Fetch all instances of parent [tag_id].
    pub async fn find_all(tag_id: u32, db: &impl ConnectionTrait) -> Result<Vec<Self>, CurdError> {
        let models = tag_enum_option::Entity::find()
            .filter(tag_enum_option::Column::TagDescriptorId.eq(tag_id))
            .filter(tag_enum_option::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        let mut v = Vec::with_capacity(models.len());
        for model in models {
            v.push(Self::from(model));
        }
        Ok(v)
    }

    /// Find instance by [id].
    pub async fn find_by_id(id: u32, db: &impl ConnectionTrait) -> Result<Self, CurdError> {
        let model = tag_enum_option::Entity::find()
            .filter(tag_enum_option::Column::Id.eq(id))
            .filter(tag_enum_option::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        match model {
            Some(model) => Ok(Self::from(model)),
            None => Err(CurdError::NotFound)?,
        }
    }
}

/// Check if [tag_option_id] belongs to [user_id]. Use this to restrict
/// access to tag options of tag which to not belong to the calling user.
pub async fn is_owner(
    tag_option_id: u32,
    user_id: u32,
    db: &impl ConnectionTrait
) -> Result<(), CurdError> {
    let rows = tag_enum_option::Entity::find()
        .find_also_related(tag_descriptor::Entity)
        .filter(tag_enum_option::Column::Id.eq(tag_option_id))
        .filter(tag_enum_option::Column::DeletedAt.is_null())
        .filter(tag_descriptor::Column::UserId.eq(user_id))
        .filter(tag_descriptor::Column::DeletedAt.is_null())
        .count(db)
        .await
        .map_err(
            |error| {
                CurdError::DbErr(error)
            }
        )?;
    if rows == 0 {
        Err(CurdError::NotFound)
    } else {
        Ok(())
    }
}

/// Builder for creating or updating a model (in the database)
pub struct CreateUpdateBuilder {
    pub order: u32,
    pub value: String,
    pub name: Option<String>,
}

impl CreateUpdateBuilder {
    /// New builder from values
    pub fn new(
        order: u32,
        value: String,
        name: Option<String>,
    ) -> Self {
        Self {
            order,
            value,
            name,
        }
    }

    /// New builder from deserialized JSON structure
    pub fn from_json(model: TagOption) -> Self {
        Self {
            order: model.order,
            value: model.value,
            name: model.name,
        }
    }

    /// Insert into database and return the new instance. It will be child of [tag_id].
    pub async fn insert(
        self,
        tag_id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<TagOption, CurdError> {
        let uuid_val = uuid::Builder::from_random_bytes(rand::random()).into_uuid();

        let model = tag_enum_option::ActiveModel {
            id: NotSet,
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            deleted_at: NotSet,
            tag_descriptor_id: Set(tag_id),
            order: Set(self.order),
            value: Set(self.value.clone()),
            uuid: Set(uuid_val.clone()),
            name: Set(self.name.clone()),
        };
        let result = tag_enum_option::Entity::insert(model)
            .exec(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;

        Ok(
            TagOption {
                id: result.last_insert_id,
                tag_id,
                order: self.order,
                display_name: match &self.name {
                    Some(value) => value.clone(),
                    None => self.value.clone(),
                },
                value: self.value,
                uuid: uuid_val.to_string(),
                name: self.name,
            }
        )
    }

    /// Update instance identified by [id] in database.
    pub async fn update(
        self,
        id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<(), CurdError> {
        let result = tag_enum_option::Entity::update_many()
            .col_expr(tag_enum_option::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(tag_enum_option::Column::Order, Expr::value(self.order))
            .col_expr(tag_enum_option::Column::Value, Expr::value(self.value))
            .col_expr(tag_enum_option::Column::Name, Expr::value(self.name))
            .filter(tag_enum_option::Column::Id.eq(id))
            .filter(tag_enum_option::Column::DeletedAt.is_null())
            .exec(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        if result.rows_affected >= 1 {
            Ok(())
        } else {
            Err(CurdError::NotFound)
        }
    }
}

/// Remove instance by [id].
pub async fn remove(id: u32, db: &impl ConnectionTrait) -> Result<(), CurdError> {
    let result = tag_enum_option::Entity::update_many()
        .col_expr(tag_enum_option::Column::DeletedAt, Expr::value(chrono::Utc::now()))
        .filter(tag_enum_option::Column::Id.eq(id))
        .filter(tag_enum_option::Column::DeletedAt.is_null())
        .exec(db)
        .await
        .map_err(
            |error| {
                CurdError::DbErr(error)
            }
        )?;
    if result.rows_affected >= 1 {
        Ok(())
    } else {
        Err(CurdError::NotFound)
    }
}
