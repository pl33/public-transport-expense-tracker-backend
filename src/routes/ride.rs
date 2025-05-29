/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rocket::{
    State,
    response::status::NoContent,
    serde::json::Json,
};
use rocket_okapi::openapi;
use super::ApiError;
use crate::fairings::Database;
use crate::request_guards::{Auth, ReadOnly, ReadWrite};
use crate::responders::PaginatedResult;
use crate::model::{ride, ride::Ride};

#[openapi(tag = "Ride")]
#[get("/ride?<page>&<size>")]
pub async fn list(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    page: Option<u64>,
    size: Option<u64>,
) -> Result<PaginatedResult<Json<Vec<Ride>>>, ApiError> {
    let count = Ride::count_all(auth.user_id, db.conn.as_ref()).await?;
    if let Some(page) = page {
        if let Some(size) = size {
            if size > 0 {
                let rides = Ride::find_all_paginated(auth.user_id, db.conn.as_ref(), page, size).await?;
                Ok(PaginatedResult::new_paginated(Json(rides), count, page, size))
            } else {
                Err(
                    ApiError::new_bad_request()
                        .with_description("Page size must be greater than zero.")
                )?
            }
        } else {
            Err(
                ApiError::new_bad_request()
                    .with_description("Pagination requested and size is not defined")
            )?
        }
    } else {
        let rides = Ride::find_all(auth.user_id, db.conn.as_ref()).await?;
        Ok(PaginatedResult::new_complete(Json(rides), Some(count)))
    }
}

#[openapi(tag = "Ride")]
#[post("/ride", data = "<ride>")]
pub async fn post(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    ride: Json<Ride>,
) -> Result<Json<Ride>, ApiError> {
    let result = ride::CreateUpdateBuilder::from_json(ride.into_inner())
        .insert(auth.user_id, db.conn.as_ref())
        .await?;
    Ok(Json(result))
}

#[openapi(tag = "Ride")]
#[get("/ride/<ride_id>")]
pub async fn get(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    ride_id: u32,
) -> Result<Json<Ride>, ApiError> {
    // First, make sure that resource belongs to the user
    ride::is_owner(ride_id, auth.user_id, db.conn.as_ref()).await?;

    let ride = Ride::find_by_id(ride_id, db.conn.as_ref()).await?;
    Ok(Json(ride))
}

#[openapi(tag = "Ride")]
#[put("/ride/<ride_id>", data = "<ride>")]
pub async fn put(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    ride_id: u32,
    ride: Json<Ride>,
) -> Result<NoContent, ApiError> {
    // First, make sure that resource belongs to the user
    ride::is_owner(ride_id, auth.user_id, db.conn.as_ref()).await?;

    ride::CreateUpdateBuilder::from_json(ride.into_inner())
        .update(ride_id, db.conn.as_ref())
        .await?;
    Ok(NoContent)
}

#[openapi(tag = "Ride")]
#[delete("/ride/<ride_id>")]
pub async fn delete(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    ride_id: u32,
) -> Result<NoContent, ApiError> {
    // First, make sure that resource belongs to the user
    ride::is_owner(ride_id, auth.user_id, db.conn.as_ref()).await?;

    ride::remove(ride_id, db.conn.as_ref()).await?;
    Ok(NoContent)
}
