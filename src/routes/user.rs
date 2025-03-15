/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rocket::serde::json::Json;
use rocket_okapi::openapi;
use super::ApiError;
use crate::request_guards::Auth;

#[openapi(tag = "User")]
#[get("/user")]
pub async fn get(auth: Auth) -> Result<Json<entity::user::Model>, ApiError> {
    todo!()
}
