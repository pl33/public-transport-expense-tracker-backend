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
use serde::{Deserialize, Serialize};
use rocket_okapi::okapi::schemars;
use rocket_okapi::openapi;
use super::ApiError;
use crate::fairings::Database;
use crate::request_guards::{Auth, ReadOnly, ReadWrite};
use crate::model::{ride, ride_tag_link, ride_tag_link::RideTagLink, tag};


#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct RideTagGetReturn {
    link: RideTagLink,
    tag: tag::Tag,
}

#[openapi(tag = "Ride")]
#[get("/ride/<ride_id>/ride_tags")]
pub async fn list(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    ride_id: u32,
) -> Result<Json<Vec<RideTagGetReturn>>, ApiError> {
    // First, make sure that resource belongs to the user
    ride::is_owner(ride_id, auth.user_id, db.conn.as_ref()).await?;

    let links = RideTagLink::find_all(ride_id, db.conn.as_ref()).await?;
    let mut result = Vec::with_capacity(links.len());
    for link in links {
        let tag = tag::Tag::find_by_id(link.tag_id(), db.conn.as_ref()).await?;
        result.push(
            RideTagGetReturn {
                link,
                tag,
            }
        );
    }
    Ok(Json(result))
}

#[openapi(tag = "Ride")]
#[get("/ride/<ride_id>/ride_tags/<tag_id>")]
pub async fn get_by_tag_id(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    ride_id: u32,
    tag_id: u32,
) -> Result<Json<RideTagGetReturn>, ApiError> {
    // First, make sure that resource belongs to the user
    ride::is_owner(ride_id, auth.user_id, db.conn.as_ref()).await?;
    tag::is_owner(tag_id, auth.user_id, db.conn.as_ref()).await?;

    let link = RideTagLink::find_by_tag_id(ride_id, tag_id, db.conn.as_ref()).await?;
    let tag = tag::Tag::find_by_id(link.tag_id(), db.conn.as_ref()).await?;
    let result = RideTagGetReturn {
        link,
        tag,
    };
    Ok(Json(result))
}

#[openapi(tag = "Ride")]
#[post("/ride/<ride_id>/ride_tags/<tag_id>", data = "<link>")]
pub async fn post_by_tag_id(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    ride_id: u32,
    tag_id: u32,
    link: Json<RideTagLink>,
) -> Result<Json<RideTagLink>, ApiError> {
    // First, make sure that resource belongs to the user
    ride::is_owner(ride_id, auth.user_id, db.conn.as_ref()).await?;
    tag::is_owner(tag_id, auth.user_id, db.conn.as_ref()).await?;

    // Prevent double use of tag ID
    match RideTagLink::find_by_tag_id(ride_id, tag_id, db.conn.as_ref()).await {
        Ok(_) => Err(ApiError::new_bad_request())?,
        Err(_) => (),
    };

    let result = ride_tag_link::CreateUpdateBuilder::from_json(link.into_inner())
        .insert(ride_id, tag_id, db.conn.as_ref())
        .await?;
    Ok(Json(result))
}

#[openapi(tag = "Ride")]
#[get("/ride_tag/<link_id>")]
pub async fn get_by_link_id(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    link_id: u32,
) -> Result<Json<RideTagGetReturn>, ApiError> {
    // First, make sure that resource belongs to the user
    ride_tag_link::is_owner(link_id, auth.user_id, db.conn.as_ref()).await?;

    let link = RideTagLink::find_by_id(link_id, db.conn.as_ref()).await?;
    let tag = tag::Tag::find_by_id(link.tag_id(), db.conn.as_ref()).await?;
    let result = RideTagGetReturn {
        link,
        tag,
    };
    Ok(Json(result))
}

#[openapi(tag = "Ride")]
#[put("/ride_tag/<link_id>", data = "<link>")]
pub async fn put(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    link_id: u32,
    link: Json<RideTagLink>,
) -> Result<NoContent, ApiError> {
    // First, make sure that resource belongs to the user
    ride_tag_link::is_owner(link_id, auth.user_id, db.conn.as_ref()).await?;

    ride_tag_link::CreateUpdateBuilder::from_json(link.into_inner())
        .update(link_id, db.conn.as_ref())
        .await?;
    Ok(NoContent)
}

#[openapi(tag = "Ride")]
#[delete("/ride_tag/<link_id>")]
pub async fn delete(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    link_id: u32,
) -> Result<NoContent, ApiError> {
    // First, make sure that resource belongs to the user
    ride_tag_link::is_owner(link_id, auth.user_id, db.conn.as_ref()).await?;

    ride_tag_link::remove(link_id, db.conn.as_ref()).await?;
    Ok(NoContent)
}
