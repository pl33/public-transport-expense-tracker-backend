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
use crate::model::{tag, tag::Tag};

#[openapi(tag = "Tag")]
#[get("/tag")]
pub async fn list(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
) -> Result<Json<Vec<Tag>>, ApiError> {
    let tags = Tag::find_all(auth.user_id, db.conn.as_ref()).await?;
    Ok(Json(tags))
}

#[openapi(tag = "Tag")]
#[post("/tag", data = "<tag>")]
pub async fn post(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    tag: Json<Tag>,
) -> Result<Json<Tag>, ApiError> {
    let result = tag::CreateUpdateBuilder::from_json(tag.into_inner())
        .insert(auth.user_id, db.conn.as_ref())
        .await?;
    Ok(Json(result))
}

#[openapi(tag = "Tag")]
#[get("/tag/<tag_id>")]
pub async fn get(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    tag_id: u32,
) -> Result<Json<Tag>, ApiError> {
    // First, make sure that tag belongs to the user
    tag::is_owner(tag_id, auth.user_id, db.conn.as_ref()).await?;

    let tag = Tag::find_by_id(tag_id, db.conn.as_ref()).await?;
    Ok(Json(tag))
}

#[openapi(tag = "Tag")]
#[put("/tag/<tag_id>", data = "<tag>")]
pub async fn put(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    tag_id: u32,
    tag: Json<Tag>,
) -> Result<NoContent, ApiError> {
    // First, make sure that tag belongs to the user
    tag::is_owner(tag_id, auth.user_id, db.conn.as_ref()).await?;

    tag::CreateUpdateBuilder::from_json(tag.into_inner())
        .update(tag_id, db.conn.as_ref())
        .await?;
    Ok(NoContent)
}

#[openapi(tag = "Tag")]
#[delete("/tag/<tag_id>")]
pub async fn delete(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    tag_id: u32,
) -> Result<NoContent, ApiError> {
    // First, make sure that tag belongs to the user
    tag::is_owner(tag_id, auth.user_id, db.conn.as_ref()).await?;

    tag::remove(tag_id, db.conn.as_ref()).await?;
    Ok(NoContent)
}
