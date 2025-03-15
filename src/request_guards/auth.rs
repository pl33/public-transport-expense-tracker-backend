/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::ops::DerefMut;
use rocket::{
    Request,
    request::{FromRequest, Outcome},
};
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::{Object, SecurityRequirement, SecurityScheme, SecuritySchemeData};
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use jwt_auth::jwt::TokenVerifier;
use crate::routes::ApiError;

/// Request Guard for authentication. It investigates the Authorization HTTP header
/// for a valid JWT. It looks up the user according to the Issuer and Subject fields
/// in the database or creates a new user if there is no hit.
pub struct Auth {
    pub issuer: String,
    pub subject: String,
}

/// JWT information
struct TokenInfo {
    pub issuer: String,
    pub subject: String,
}

/// Retrieve auth cache from Rocket state
fn get_auth_cache<'r>(request: &'r Request<'_>) -> Result<&'r crate::fairings::AuthCache, ApiError> {
    Ok(
        request
            .rocket()
            .state()
            .ok_or(
                ApiError::new_internal_server_error()
                    .with_description("Cannot retrieve auth cache from Rocket state")
            )?
    )
}

/// Validate bearer and extract JWT information
fn validate_bearer(request: &Request<'_>, bearer: &str) -> Result<TokenInfo, ApiError> {
    let auth_cache = get_auth_cache(request)?;
    let mut key_cache = auth_cache
        .key_cache
        .write()
        .map_err(|e| {
            ApiError::new_internal_server_error()
                .with_description(format!("Key cache lock poison error, {}", e.to_string()))
        })?;
    let mut verifier = TokenVerifier::new(key_cache.deref_mut())
        .expect_audience(&auth_cache.expect_jwt_audience)
        .with_max_expiration(auth_cache.jwt_max_expiration);
    if let Some(expect_jwt_issuer) = &auth_cache.expect_jwt_issuer {
        verifier = verifier.expect_issuer(expect_jwt_issuer);
    }
    if let Some(issued_after) = auth_cache.jwt_issued_after {
        verifier = verifier.must_be_issued_after(issued_after);
    }
    match verifier.verify(bearer)
    {
        Ok((token, _)) => {
            let issuer = match &token.claims().registered.issuer {
                Some(issuer) => issuer.clone(),
                None => Err(
                    ApiError::new_bad_request()
                        .with_description("Issuer is not set in token")
                )?,
            };
            let subject = match &token.claims().registered.subject {
                Some(subject) => subject.clone(),
                None => Err(
                    ApiError::new_bad_request()
                        .with_description("Subject is not set in token")
                )?,
            };
            Ok(
                TokenInfo {
                    issuer,
                    subject,
                }
            )
        },
        Err(err) => Err(
            ApiError::new_unauthorized()
                .with_description(err.to_string())
                .into()
        ),
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Auth {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(auth) = request.headers().get_one("Authorization") {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];
                match validate_bearer(request, token) {
                    Ok(token) => Outcome::Success(
                        Auth {
                            issuer: token.issuer,
                            subject: token.subject,
                        }
                    ),
                    Err(err) => Outcome::Error(err.into()),
                }
            } else {
                Outcome::Error(
                    ApiError::new_bad_request()
                        .with_description("Authorization must be Bearer")
                        .into()
                )
            }
        } else {
            Outcome::Error(
                ApiError::new_bad_request()
                    .with_description("Authorization header is missing")
                    .into()
            )
        }
    }
}

impl OpenApiFromRequest<'_> for Auth {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        Ok(
            RequestHeaderInput::Security(
                "BearerAuth".to_string(),
                SecurityScheme{
                    description: Some(
                        "JWT is required for authentication".to_string()
                    ),
                    data: SecuritySchemeData::Http {
                        scheme: "bearer".to_string(),
                        bearer_format: Some(
                            "<JSON Web Token (JWT)>".to_string()
                        )
                    },
                    extensions: Object::new(),
                },
                SecurityRequirement::from(
                    [
                        (
                            "BearerAuth".to_string(),
                            Vec::new(),
                        ),
                    ]
                ),
            )
        )
    }
}
