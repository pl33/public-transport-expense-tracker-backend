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
use crate::model::{ride, ride::Ride};

#[openapi(tag = "Ride")]
#[get("/ride")]
pub async fn list(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
) -> Result<Json<Vec<Ride>>, ApiError> {
    let rides = Ride::find_all(auth.user_id, db.conn.as_ref()).await?;
    Ok(Json(rides))
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
