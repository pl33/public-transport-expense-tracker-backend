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
use entity::ride;
use entity::ride_tag;
use entity::tag_descriptor::TagType;
use super::error::CurdError;
use super::tag::Tag;

/// JSON structure
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RideTagLink {
    #[serde(skip_deserializing)]
    id: u32,
    #[serde(skip_deserializing)]
    ride_id: u32,
    #[serde(skip_deserializing)]
    tag_id: u32,
    pub order: u32,
    pub value: Value,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    DateTime(DateTimeUtc),
    EnumOption(u32),
}

impl Value {
    pub fn validate(&self, tag: &Tag) -> Result<(), &'static str> {
        let tag_type = TagType::try_from(tag.tag_type.clone())
            .map_err(
                |_| {
                    "Invalid tag type stored in tag"
                }
            )?;
        match self {
            Self::Integer(_) => {
                if tag_type != TagType::Integer {
                    Err("Expected integer value in link {}")?
                }
            },
            Self::Float(_) => {
                if tag_type != TagType::Float {
                    Err("Expected float value in link")?
                }
            },
            Self::String(_) => {
                if tag_type != TagType::String {
                    Err("Expected string value in link")?
                }
            },
            Self::DateTime(_) => {
                if tag_type != TagType::DateTime {
                    Err("Expected date/time value in link")?
                }
            },
            Self::EnumOption(option_id) => {
                if tag_type != TagType::Enum {
                    Err("Expected Option ID in link")?
                }
                if !tag.has_option_id(*option_id) {
                    Err("Option ID does not belong to the tag")?
                }
            },
        }
        Ok(())
    }
}

impl TryFrom<ride_tag::Model> for RideTagLink {
    type Error = CurdError;
    
    fn try_from(model: ride_tag::Model) -> Result<Self, Self::Error> {
        RideTagLink::from_model(model)
    }
}

impl RideTagLink {
    /// Getter for [id]
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Getter for [ride_id]
    pub fn ride_id(&self) -> u32 {
        self.ride_id
    }

    /// Getter for [tag_id]
    pub fn tag_id(&self) -> u32 {
        self.tag_id
    }

    pub fn from_model(model: ride_tag::Model) -> Result<Self, CurdError> {
        let value = if let Some(value) = &model.value_integer {
            Value::Integer(*value)
        } else if let Some(value) = &model.value_float {
            Value::Float(*value)
        } else if let Some(value) = &model.value_string {
            Value::String(value.to_string())
        } else if let Some(value) = &model.value_date_time {
            Value::DateTime(*value)
        } else if let Some(value) = &model.value_enum_option_id {
            Value::EnumOption(*value)
        } else {
            Err(CurdError::InternalError(format!("Cannot infer value type from {}", model.id)))?
        };
        let link = Self {
            id: model.id,
            ride_id: model.ride_id,
            tag_id: model.tag_descriptor_id,
            order: model.order,
            value,
            remarks: model.remarks,
        };
        Ok(link)
    }

    /// Fetch all instances belonging to [ride_id]
    pub async fn find_all(ride_id: u32, db: &impl ConnectionTrait) -> Result<Vec<Self>, CurdError> {
        let models = ride_tag::Entity::find()
            .filter(ride_tag::Column::RideId.eq(ride_id))
            .filter(ride_tag::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        let mut result = Vec::with_capacity(models.len());
        for model in models {
            result.push(Self::try_from(model)?);
        }
        Ok(result)
    }

    /// Find instance by [tag_id] of [ride_id].
    pub async fn find_by_tag_id(ride_id: u32, tag_id: u32, db: &impl ConnectionTrait) -> Result<Self, CurdError> {
        let mut model = ride_tag::Entity::find()
            .filter(ride_tag::Column::RideId.eq(ride_id))
            .filter(ride_tag::Column::TagDescriptorId.eq(tag_id))
            .filter(ride_tag::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        match model {
            Some(model) => Ok(Self::from_model(model)?),
            None => Err(CurdError::NotFound)?,
        }
    }

    /// Find instance by [id].
    pub async fn find_by_id(id: u32, db: &impl ConnectionTrait) -> Result<Self, CurdError> {
        let mut model = ride_tag::Entity::find()
            .filter(ride_tag::Column::Id.eq(id))
            .filter(ride_tag::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        match model {
            Some(model) => Ok(Self::try_from(model)?),
            None => Err(CurdError::NotFound)?,
        }
    }
}

/// Check if [link_id] belongs to [user_id]. Use this to restrict
/// access to tag options of tag which to not belong to the calling user.
pub async fn is_owner(
    link_id: u32,
    user_id: u32,
    db: &impl ConnectionTrait
) -> Result<(), CurdError> {
    let rows = ride_tag::Entity::find()
        .filter(ride_tag::Column::Id.eq(link_id))
        .filter(ride_tag::Column::DeletedAt.is_null())
        .filter(ride::Column::UserId.eq(user_id))
        .filter(ride::Column::DeletedAt.is_null())
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
    pub value: Value,
    pub remarks: Option<String>,
}

impl CreateUpdateBuilder {
    /// New builder from values
    pub fn new(
        order: u32,
        value: Value,
        remarks: Option<String>,
    ) -> Self {
        Self {
            order,
            value,
            remarks,
        }
    }

    /// New builder from deserialized JSON structure
    pub fn from_json(model: RideTagLink) -> Self {
        Self {
            order: model.order,
            value: model.value,
            remarks: model.remarks,
        }
    }

    fn get_value_integer(&self) -> Option<i64> {
        if let Value::Integer(value) = self.value {
            Some(value)
        } else {
            None
        }
    }

    fn get_value_float(&self) -> Option<f64> {
        if let Value::Float(value) = &self.value {
            Some(*value)
        } else {
            None
        }
    }

    fn get_value_string(&self) -> Option<String> {
        if let Value::String(value) = &self.value {
            Some(value.to_string())
        } else {
            None
        }
    }

    fn get_value_date_time(&self) -> Option<DateTimeUtc> {
        if let Value::DateTime(value) = &self.value {
            Some(*value)
        } else {
            None
        }
    }

    fn get_value_enum_option_id(&self) -> Option<u32> {
        if let Value::EnumOption(value) = &self.value {
            Some(*value)
        } else {
            None
        }
    }

    /// Insert into database and return the new instance. It will belong to [ride_id] and [tag_id].
    pub async fn insert(
        self,
        ride_id: u32,
        tag_id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<RideTagLink, CurdError> {
        let model = ride_tag::ActiveModel {
            id: NotSet,
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            deleted_at: NotSet,
            ride_id: Set(ride_id),
            tag_descriptor_id: Set(tag_id),
            order: Set(self.order),
            value_integer: Set(self.get_value_integer()),
            value_float: Set(self.get_value_float()),
            value_string: Set(self.get_value_string()),
            value_date_time: Set(self.get_value_date_time()),
            value_enum_option_id: Set(self.get_value_enum_option_id()),
            remarks: Set(self.remarks.clone()),
        };
        let result = ride_tag::Entity::insert(model)
            .exec(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;

        Ok(
            RideTagLink {
                id: result.last_insert_id,
                ride_id,
                tag_id,
                order: self.order,
                value: self.value,
                remarks: self.remarks,
            }
        )
    }

    /// Update instance identified by [id] in database.
    pub async fn update(
        self,
        id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<(), CurdError> {
        let result = ride_tag::Entity::update_many()
            .col_expr(ride_tag::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(ride_tag::Column::Order, Expr::value(self.order))
            .col_expr(ride_tag::Column::ValueInteger, Expr::value(self.get_value_integer()))
            .col_expr(ride_tag::Column::ValueFloat, Expr::value(self.get_value_float()))
            .col_expr(ride_tag::Column::ValueString, Expr::value(self.get_value_string()))
            .col_expr(ride_tag::Column::ValueDateTime, Expr::value(self.get_value_date_time()))
            .col_expr(ride_tag::Column::ValueEnumOptionId, Expr::value(self.get_value_enum_option_id()))
            .col_expr(ride_tag::Column::Remarks, Expr::value(self.remarks.clone()))
            .filter(ride::Column::Id.eq(id))
            .filter(ride::Column::DeletedAt.is_null())
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
    let result = ride_tag::Entity::update_many()
        .col_expr(ride_tag::Column::DeletedAt, Expr::value(chrono::Utc::now()))
        .filter(ride_tag::Column::Id.eq(id))
        .filter(ride_tag::Column::DeletedAt.is_null())
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
