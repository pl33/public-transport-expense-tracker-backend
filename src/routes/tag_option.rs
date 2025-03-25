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
use crate::model::{tag, tag_option, tag_option::TagOption};

#[openapi(tag = "Tag")]
#[get("/tag/<tag_id>/tag_option")]
pub async fn list(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    tag_id: u32,
) -> Result<Json<Vec<TagOption>>, ApiError> {
    // First, make sure that tag belongs to the user
    tag::is_owner(tag_id, auth.user_id, db.conn.as_ref()).await?;

    let tags = TagOption::find_all(tag_id, db.conn.as_ref()).await?;
    Ok(Json(tags))
}

#[openapi(tag = "Tag")]
#[post("/tag/<tag_id>/tag_option", data = "<option>")]
pub async fn post(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    tag_id: u32,
    option: Json<TagOption>,
) -> Result<Json<TagOption>, ApiError> {
    // First, make sure that tag belongs to the user
    tag::is_owner(tag_id, auth.user_id, db.conn.as_ref()).await?;

    let result = tag_option::CreateUpdateBuilder::from_json(option.into_inner())
        .insert(tag_id, db.conn.as_ref())
        .await?;
    Ok(Json(result))
}

#[openapi(tag = "Tag")]
#[get("/tag_option/<option_id>")]
pub async fn get(
    auth: Auth<ReadOnly>,
    db: &State<Database>,
    option_id: u32,
) -> Result<Json<TagOption>, ApiError> {
    // First, make sure that tag option belongs to the user
    tag_option::is_owner(option_id, auth.user_id, db.conn.as_ref()).await?;

    let tag = TagOption::find_by_id(option_id, db.conn.as_ref()).await?;
    Ok(Json(tag))
}

#[openapi(tag = "Tag")]
#[put("/tag_option/<option_id>", data = "<option>")]
pub async fn put(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    option_id: u32,
    option: Json<TagOption>,
) -> Result<NoContent, ApiError> {
    // First, make sure that tag option belongs to the user
    tag_option::is_owner(option_id, auth.user_id, db.conn.as_ref()).await?;

    tag_option::CreateUpdateBuilder::from_json(option.into_inner())
        .update(option_id, db.conn.as_ref())
        .await?;
    Ok(NoContent)
}

#[openapi(tag = "Tag")]
#[delete("/tag_option/<option_id>")]
pub async fn delete(
    auth: Auth<ReadWrite>,
    db: &State<Database>,
    option_id: u32,
) -> Result<NoContent, ApiError> {
    // First, make sure that tag option belongs to the user
    tag_option::is_owner(option_id, auth.user_id, db.conn.as_ref()).await?;

    tag_option::remove(option_id, db.conn.as_ref()).await?;
    Ok(NoContent)
}
