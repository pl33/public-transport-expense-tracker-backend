/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rocket::http::Status;
use serde::{Serialize, Deserialize};
use rocket_okapi::{
    response::OpenApiResponderInner,
    okapi::schemars,
};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
pub struct ErrorInfo {
    /// HTTP status code
    code: u16,
    /// Error reason
    reason: String,
    /// Detailed description
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, schemars::JsonSchema)]
pub struct ApiError {
    /// Details about the error
    error: ErrorInfo,
}

impl ApiError {
    pub fn new_not_found() -> Self {
        ApiError {
            error: ErrorInfo {
                code: Status::NotFound.code,
                reason: "Not found".to_string(),
                description: None,
            },
        }
    }

    pub fn new_unauthorized() -> Self {
        ApiError {
            error: ErrorInfo {
                code: Status::Unauthorized.code,
                reason: "Unauthorized".to_string(),
                description: None,
            },
        }
    }

    pub fn new_bad_request() -> Self {
        ApiError {
            error: ErrorInfo {
                code: Status::BadRequest.code,
                reason: "Bad Request".to_string(),
                description: None,
            },
        }
    }

    pub fn new_internal_server_error() -> Self {
        ApiError {
            error: ErrorInfo {
                code: Status::InternalServerError.code,
                reason: "Internal Server Error".to_string(),
                description: None,
            },
        }
    }

    pub fn with_description<S: ToString>(mut self, description: S) -> Self {
        self.error.description = Some(description.to_string());
        self
    }

    pub fn to_status(&self) -> Status {
        Status::from_code(self.error.code).unwrap_or(rocket::http::Status::InternalServerError)
    }
}

impl Into<(Status, ApiError)> for ApiError {
    fn into(self) -> (Status, ApiError) {
        (self.to_status(), self)
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(value: sea_orm::DbErr) -> Self {
        ApiError::new_internal_server_error()
            .with_description(value.to_string())
    }
}

impl<'r> rocket::response::Responder<'r, 'static> for ApiError {
    fn respond_to(self, _: &'r rocket::Request) -> rocket::response::Result<'static> {
        let body = serde_json::to_string(&self).unwrap();
        rocket::Response::build()
            .sized_body(body.len(), std::io::Cursor::new(body))
            .header(rocket::http::ContentType::JSON)
            .status(Status::new(self.error.code))
            .ok()
    }
}

impl OpenApiResponderInner for ApiError {
    fn responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        use rocket_okapi::okapi::{map, openapi3::{RefOr, MediaType}};
        let mut make_response = |description: &str| {
            rocket_okapi::okapi::openapi3::Response {
                description: description.to_string(),
                content: map! {
                    "application/json".to_owned() => MediaType {
                        schema: Some(gen.json_schema::<ApiError>()),
                        ..Default::default()
                    }
                },
                ..Default::default()
            }
        };
        Ok(Responses {
            responses: map! {
                "400".to_owned() => RefOr::Object(make_response("Bad Request")),
                "401".to_owned() => RefOr::Object(make_response("Unauthorized")),
                "404".to_owned() => RefOr::Object(make_response("Not Found")),
                "500".to_owned() => RefOr::Object(make_response("Internal Server Error")),
            },
            ..Default::default()
        })
    }
}
