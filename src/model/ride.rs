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
use super::error::CurdError;
use super::ride_tag_link::RideTagLink;

/// JSON structure
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Ride {
    #[serde(skip_deserializing)]
    id: u32,
    pub journey_departure: DateTimeUtc,
    pub journey_arrival: Option<DateTimeUtc>,
    pub location_from: String,
    pub location_to: String,
    pub remarks: Option<String>,
    pub is_template: bool,
    #[serde(skip_deserializing)]
    tags: Vec<RideTagLink>,
}

impl Ride {
    /// Getter for [id]
    pub fn id(&self) -> u32 {
        self.id
    }

    fn from_models(ride: ride::Model, tags: Vec<ride_tag::Model>) -> Result<Self, CurdError> {
        let tags = {
            let mut option_arr = Vec::with_capacity(tags.len());
            for tag in tags {
                option_arr.push(RideTagLink::try_from(tag)?);
            }
            option_arr
        };

        let ride = Self {
            id: ride.id,
            journey_departure: ride.journey_departure,
            journey_arrival: ride.journey_arrival,
            location_from: ride.location_from,
            location_to: ride.location_to,
            remarks: ride.remarks,
            is_template: ride.is_template,
            tags,
        };
        Ok(ride)
    }

    /// Fetch all instances belonging to [user_id]
    pub async fn find_all(user_id: u32, db: &impl ConnectionTrait) -> Result<Vec<Self>, CurdError> {
        let models = ride::Entity::find()
            .find_with_related(ride_tag::Entity)
            .filter(ride::Column::UserId.eq(user_id))
            .filter(ride::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        let mut result = Vec::with_capacity(models.len());
        for (tag, options) in models {
            result.push(Self::from_models(tag, options)?);
        }
        Ok(result)
    }

    /// Find instance by [id].
    pub async fn find_by_id(id: u32, db: &impl ConnectionTrait) -> Result<Self, CurdError> {
        let mut model = ride::Entity::find()
            .find_with_related(ride_tag::Entity)
            .filter(ride::Column::Id.eq(id))
            .filter(ride::Column::DeletedAt.is_null())
            .all(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;
        match model.pop() {
            Some((tag, options)) => Ok(Self::from_models(tag, options)?),
            None => Err(CurdError::NotFound)?,
        }
    }
}

/// Check if [tag_id] belongs to [user_id]. Use this to restrict
/// access to tag options of tag which to not belong to the calling user.
pub async fn is_owner(
    ride_id: u32,
    user_id: u32,
    db: &impl ConnectionTrait
) -> Result<(), CurdError> {
    let rows = ride::Entity::find()
        .filter(ride::Column::Id.eq(ride_id))
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
    pub journey_departure: DateTimeUtc,
    pub journey_arrival: Option<DateTimeUtc>,
    pub location_from: String,
    pub location_to: String,
    pub remarks: Option<String>,
    pub is_template: bool,
}

impl CreateUpdateBuilder {
    /// New builder from values
    pub fn new(
        journey_departure: DateTimeUtc,
        journey_arrival: Option<DateTimeUtc>,
        location_from: String,
        location_to: String,
        remarks: Option<String>,
        is_template: bool,
    ) -> Self {
        Self {
            journey_departure,
            journey_arrival,
            location_from,
            location_to,
            remarks,
            is_template,
        }
    }

    /// New builder from deserialized JSON structure
    pub fn from_json(model: Ride) -> Self {
        Self {
            journey_departure: model.journey_departure,
            journey_arrival: model.journey_arrival,
            location_from: model.location_from,
            location_to: model.location_to,
            remarks: model.remarks,
            is_template: model.is_template,
        }
    }

    /// Insert into database and return the new instance. It will belong to [user_id].
    pub async fn insert(
        self,
        user_id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<Ride, CurdError> {
        let model = ride::ActiveModel {
            id: NotSet,
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            deleted_at: NotSet,
            user_id: Set(user_id),
            journey_departure: Set(self.journey_departure.clone()),
            journey_arrival: Set(self.journey_arrival.clone()),
            location_from: Set(self.location_from.clone()),
            location_to: Set(self.location_to.clone()),
            remarks: Set(self.remarks.clone()),
            is_template: Set(self.is_template),
        };
        let result = ride::Entity::insert(model)
            .exec(db)
            .await
            .map_err(
                |error| {
                    CurdError::DbErr(error)
                }
            )?;

        Ok(
            Ride {
                id: result.last_insert_id,
                journey_departure: self.journey_departure,
                journey_arrival: self.journey_arrival,
                location_from: self.location_from,
                location_to: self.location_to,
                remarks: self.remarks,
                is_template: self.is_template,
                tags: Vec::new(),
            }
        )
    }

    /// Update instance identified by [id] in database.
    pub async fn update(
        self,
        id: u32,
        db: &impl ConnectionTrait,
    ) -> Result<(), CurdError> {
        let result = ride::Entity::update_many()
            .col_expr(ride::Column::UpdatedAt, Expr::value(chrono::Utc::now()))
            .col_expr(ride::Column::JourneyDeparture, Expr::value(self.journey_departure.clone()))
            .col_expr(ride::Column::JourneyArrival, Expr::value(self.journey_arrival.clone()))
            .col_expr(ride::Column::LocationFrom, Expr::value(self.location_from.clone()))
            .col_expr(ride::Column::LocationTo, Expr::value(self.location_to.clone()))
            .col_expr(ride::Column::Remarks, Expr::value(self.remarks.clone()))
            .col_expr(ride::Column::IsTemplate, Expr::value(self.is_template))
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
    let result = ride::Entity::update_many()
        .col_expr(ride::Column::DeletedAt, Expr::value(chrono::Utc::now()))
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
