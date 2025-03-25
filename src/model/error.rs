/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt::Display;
use sea_orm::error::DbErr;
use crate::routes::ApiError;

/// Errors of CURD operations
pub enum CurdError {
    NotFound,
    DeserializationError(String),
    DbErr(DbErr),
    InternalError(String),
}

impl From<CurdError> for ApiError {
    fn from(e: CurdError) -> ApiError {
        match e {
            CurdError::NotFound => ApiError::new_not_found(),
            CurdError::DbErr(e) => {
                ApiError::new_internal_server_error()
                    .with_description(e.to_string())
            },
            CurdError::DeserializationError(e) => {
                ApiError::new_bad_request()
                    .with_description(e)
            },
            CurdError::InternalError(e) => {
                ApiError::new_internal_server_error()
                    .with_description(e)
            },
        }
    }
}

impl Display for CurdError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CurdError::NotFound => write!(f, "Not found"),
            CurdError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            CurdError::DbErr(e) => write!(f, "Db error: {}", e),
            CurdError::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}
