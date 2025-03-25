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
};
use rand;
use uuid;
use entity::tag_descriptor;
use entity::tag_enum_option;
use super::error::CurdError;
use super::tag_option::TagOption;

/// JSON structure
#[derive(Debug, Clone, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Tag {
    #[serde(skip_deserializing)]
    id: u32,
    pub tag_type: String,
    tag_key: String,
    tag_name: Option<String>,
    #[serde(skip_deserializing)]
    tag_display_name: String,
    #[serde(skip_deserializing)]
    uuid: String,
    pub unit: Option<String>,
    pub remarks: Option<String>,
    #[serde(skip_deserializing)]
    options: Option<Vec<TagOption>>,
}

impl From<tag_descriptor::Model> for Tag {
    fn from(model: tag_descriptor::Model) -> Self {
        Self {
            id: model.id,
            tag_type: model.tag_type.into(),
            tag_display_name: match &model.tag_name {
                Some(value) => value.clone(),
                None => model.tag_key.clone(),
            },
            tag_key: model.tag_key,
            tag_name: model.tag_name,
            uuid: model.uuid.to_string(),
            unit: model.unit,
            remarks: model.remarks,
            options: None,
        }
    }
}

impl Tag {
    /// Getter for [id]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Getter for [tag_key]
    pub fn tag_key(&self) -> &String {
        &self.tag_key
    }

    /// Setter for [tag_key]
    pub fn set_tag_key<S: ToString>(&mut self, value: S) {
        self.tag_key = value.to_string();
    }

    /// Getter for [tag_name]
    pub fn tag_name(&self) -> &Option<String> {
        &self.tag_name
    }

    /// Setter for [tag_name] and [tag_display_name]
    pub fn set_tag_name<S: ToString>(&mut self, value: Option<S>) {
        self.tag_name = match &value {
            Some(v) => Some(v.to_string()),
            None => None,
        };
        self.tag_display_name = match &value {
            Some(v) => v.to_string(),
            None => self.tag_key.clone(),
        };
    }

    /// Getter for [tag_display_name]
    pub fn tag_display_name(&self) -> &String {
        &self.tag_display_name
    }

    /// Getter for [uuid]
    pub fn uuid(&self) -> &String {
        &self.uuid
    }

    /// Checks if [option_id] is in options array
    pub fn has_option_id(&self, option_id: u32) -> bool {
        match &self.options {
            Some(options) => {
                options.iter().any(|option| { option.id() == option_id })
            },
            None => false,
        }
    }

    fn from_models(tag: tag_descriptor::Model, options: Vec<tag_enum_option::Model>) -> Self {
        let mut tag = Self::from(tag);
        if tag.tag_type == "enum" {
            tag.options = Some(
                {
                    let mut option_arr = Vec::with_capacity(options.len());
                    for option in options {
                        option_arr.push(TagOption::from(option));
                    }
                    option_arr
                }
            );
        }
        tag
    }

    /// Fetch all instances belonging to [user_id]
    pub async fn find_all(user_id: u32, db: &impl ConnectionTrait) -> Result<Vec<Self>, CurdError> {
        let models = tag_descriptor::Entity::find()
            .find_with_related(tag_enum_option::Entity)
            .filter(tag_descriptor::Column::UserId.eq(user_id))
            .filter(tag_descriptor::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        let mut result = Vec::with_capacity(models.len());
        for (tag, options) in models {
            result.push(Self::from_models(tag, options));
        }
        Ok(result)
    }

    /// Find instance by [id].
    pub async fn find_by_id(id: u32, db: &impl ConnectionTrait) -> Result<Self, CurdError> {
        let mut model = tag_descriptor::Entity::find()
            .find_with_related(tag_enum_option::Entity)
            .filter(tag_descriptor::Column::Id.eq(id))
            .filter(tag_descriptor::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        match model.pop() {
            Some((tag, options)) => Ok(Self::from_models(tag, options)),
            None => Err(CurdError::NotFound)?,
        }
    }
}

/// Check if [tag_id] belongs to [user_id]. Use this to restrict
/// access to tag options of tag which to not belong to the calling user.
pub async fn is_owner(
    tag_id: u32,
    user_id: u32,
    db: &impl ConnectionTrait
) -> Result<(), CurdError> {
    let rows = tag_descriptor::Entity::find()
        .filter(tag_descriptor::Column::Id.eq(tag_id))
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
pub struct CreateUpdateBuilder<T: TryInto<tag_descriptor::TagType>> where T::Error: ToString {
    pub tag_type: T,
    pub tag_key: String,
    pub tag_name: Option<String>,
    pub unit: Option<String>,
    pub remarks: Option<String>,
}

impl CreateUpdateBuilder<String> {
    /// New builder from deserialized JSON structure
    pub fn from_json(model: Tag) -> Self {
        Self {
            tag_type: model.tag_type,
            tag_key: model.tag_key,
            tag_name: model.tag_name,
            unit: model.unit,
            remarks: model.remarks,
        }
    }
}

impl<T: TryInto<tag_descriptor::TagType>> CreateUpdateBuilder<T> where T::Error: ToString {
    /// New builder from values
    pub fn new(
        tag_type: T,
        tag_key: String,
        tag_name: Option<String>,
        unit: Option<String>,
        remarks: Option<String>,
    ) -> Self {
        Self {
            tag_type,
            tag_key,
            tag_name,
            unit,
            remarks,
        }
    }

    /// Insert into database and return the new instance. It will belong to [user_id].
    pub async fn insert(
        self,
        user_id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<Tag, CurdError> {
        let uuid_val = uuid::Builder::from_random_bytes(rand::random()).into_uuid();
        let tag_type: tag_descriptor::TagType = match self.tag_type.try_into() {
            Ok(value) => value,
            Err(e) => Err(CurdError::DeserializationError(e.to_string()))?,
        };

        let model = tag_descriptor::ActiveModel {
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            user_id: Set(user_id),
            tag_type: Set(tag_type.clone()),
            tag_key: Set(self.tag_key.clone()),
            tag_name: Set(self.tag_name.clone()),
            uuid: Set(uuid_val.clone()),
            unit: Set(self.unit.clone()),
            remarks: Set(self.remarks.clone()),
            ..Default::default()
        };
        let result = tag_descriptor::Entity::insert(model)
            .exec(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;

        Ok(
            Tag {
                id: result.last_insert_id,
                tag_type: tag_type.into(),
                tag_display_name: match &self.tag_name {
                    Some(value) => value.clone(),
                    None => self.tag_key.clone(),
                },
                tag_key: self.tag_key,
                tag_name: self.tag_name,
                uuid: uuid_val.to_string(),
                unit: self.unit,
                remarks: self.remarks,
                options: None,
            }
        )
    }

    /// Update instance identified by [id] in database.
    pub async fn update(
        self,
        id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<(), CurdError> {
        let result = tag_descriptor::Entity::update_many()
            .col_expr(tag_descriptor::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(
                tag_descriptor::Column::TagType,
                Expr::value(
                    match self.tag_type.try_into() {
                        Ok(value) => value,
                        Err(e) => Err(CurdError::DeserializationError(e.to_string()))?,
                    }
                )
            )
            .col_expr(tag_descriptor::Column::TagKey, Expr::value(self.tag_key.clone()))
            .col_expr(tag_descriptor::Column::TagName, Expr::value(self.tag_name.clone()))
            .col_expr(tag_descriptor::Column::Unit, Expr::value(self.unit.clone()))
            .col_expr(tag_descriptor::Column::Remarks, Expr::value(self.remarks.clone()))
            .filter(tag_descriptor::Column::Id.eq(id))
            .filter(tag_descriptor::Column::DeletedAt.is_null())
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
    let result = tag_descriptor::Entity::update_many()
        .col_expr(tag_descriptor::Column::DeletedAt, Expr::value(chrono::Utc::now()))
        .filter(tag_descriptor::Column::Id.eq(id))
        .filter(tag_descriptor::Column::DeletedAt.is_null())
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
