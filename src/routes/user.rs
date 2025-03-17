/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rocket::{State, serde::json::Json};
use rocket_okapi::openapi;
use sea_orm::prelude::*;
use sea_orm::{Set, IntoActiveModel};
use entity::user::{Model as UserModel, Entity as UserEntity, Column as UserColumn, ActiveModel as UserActiveModel};
use super::ApiError;
use crate::fairings::Database;
use crate::request_guards::{Auth, ReadOnly, ReadWrite};

async fn find_user_by_id(id: u32, db: &impl ConnectionTrait) -> Result<Option<UserModel>, ApiError> {
    Ok(
        UserEntity::find()
            .filter(UserColumn::Id.eq(id))
            .one(db)
            .await
            .map_err(
                |e| {
                    ApiError::from(e)
                }
            )?
    )
}

#[openapi(tag = "User")]
#[get("/user")]
pub async fn get(auth: Auth<ReadOnly>, db: &State<Database>) -> Result<Json<UserModel>, ApiError> {
    match find_user_by_id(auth.user_id, db.conn.as_ref()).await? {
        Some(user) => Ok(Json(user)),
        None => Err(
            ApiError::new_internal_server_error()
        )
    }
}

#[openapi(tag = "User")]
#[put("/user", data = "<user>")]
pub async fn put(auth: Auth<ReadWrite>, db: &State<Database>, user: Json<UserModel>) -> Result<Json<UserModel>, ApiError> {
    let mut model = match find_user_by_id(auth.user_id, db.conn.as_ref()).await? {
        Some(model) => model.into_active_model(),
        None => Err(
            ApiError::new_internal_server_error()
        )?
    };
    model.name = Set(user.name.clone());
    match model.update(db.conn.as_ref()).await {
        Ok(model) => Ok(Json(model)),
        Err(e) => Err(ApiError::from(e))
    }
}
