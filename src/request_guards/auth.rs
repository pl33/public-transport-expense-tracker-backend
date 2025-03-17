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
use sea_orm::{prelude::*, ActiveValue::Set};
use jwt_auth::jwt::TokenVerifier;
use crate::routes::ApiError;
use crate::fairings::auth_cache::TokenInfo;

/// Request Guard for authentication. It investigates the Authorization HTTP header
/// for a valid JWT. It looks up the user according to the Issuer and Subject fields
/// in the database or creates a new user if there is no hit.
pub struct Auth<Val: JwtValidator> {
    jwt_validator: Val,
    /// ID of the user in the database
    pub user_id: u32,
}

/// Validate the JSON Web Token
pub trait JwtValidator: Sized + Send {
    /// Validate the claims of a JSON Web Token
    fn validate(claims: &serde_json::Value) -> Result<Self, String>;
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

/// Retrieve DB from Rocket state
fn get_db<'r>(request: &'r Request<'_>) -> Result<&'r crate::fairings::Database, ApiError> {
    Ok(
        request
            .rocket()
            .state()
            .ok_or(
                ApiError::new_internal_server_error()
                    .with_description("Cannot retrieve DB from Rocket state")
            )?
    )
}

async fn lookup_or_make_user<'r>(request: &'r Request<'_>, token: &TokenInfo) -> Result<u32, ApiError> {
    use entity::user::{Entity as UserEntity, Column as UserColumn, ActiveModel as UserActiveModel};

    let auth_cache = get_auth_cache(request)?;
    let mut model_cache = auth_cache
        .user_model_cache
        .write()
        .await;

    let user_id = match model_cache.get(token) {
        Some(id) => *id,
        None => {
            let db = get_db(request)?;

            let user = UserEntity::find()
                .filter(UserColumn::JwtIssuer.eq(token.issuer.as_str()))
                .filter(UserColumn::JwtSubject.eq(token.subject.as_str()))
                .one(db.conn.as_ref())
                .await
                .map_err(|db_err| {
                    ApiError::from(db_err)
                })?;
            match user {
                Some(user) => {
                    model_cache.insert(token.clone(), user.id);
                    user.id
                },
                None => {
                    let model = UserActiveModel {
                        jwt_issuer: Set(token.issuer.clone()),
                        jwt_subject: Set(token.subject.clone()),
                        name: Set(None),
                        ..Default::default()
                    };
                    let model = model
                        .insert(db.conn.as_ref())
                        .await
                        .map_err(|db_err| {
                            ApiError::from(db_err)
                        })?;
                    model.id
                },
            }
        }
    };

    Ok(user_id)
}

/// Validate bearer and extract JWT information
async fn validate_bearer(
    request: &Request<'_>,
    bearer: &str,
) -> Result<(TokenInfo, serde_json::Value), ApiError> {
    let auth_cache = get_auth_cache(request)?;
    let mut key_cache = auth_cache
        .key_cache
        .write()
        .await;
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
            let claims = serde_json::to_value(token.claims())
                .map_err(
                    |e| {
                        ApiError::new_internal_server_error()
                            .with_description(e.to_string())
                    }
                )?;
            Ok(
                (
                    TokenInfo {
                        issuer,
                        subject,
                    },
                    claims,
                )
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
impl<'r, Val: JwtValidator> FromRequest<'r> for Auth<Val> {
    type Error = ApiError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        if let Some(auth) = request.headers().get_one("Authorization") {
            if auth.starts_with("Bearer ") {
                let token = &auth[7..];
                match validate_bearer(request, token).await {
                    Ok((token, claims)) => {
                        match Val::validate(&claims) {
                            Ok(val) => match lookup_or_make_user(request, &token).await {
                                Ok(user_id) => Outcome::Success(Auth { jwt_validator: val, user_id }),
                                Err(err) => Outcome::Error(err.into()),
                            },
                            Err(e) => Outcome::Error(
                                ApiError::new_unauthorized()
                                    .with_description(e.to_string())
                                    .into()
                            )
                        }
                    },
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

impl<Val: JwtValidator> OpenApiFromRequest<'_> for Auth<Val> {
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

/// Validates that a token grants read-only access
pub struct ReadOnly {}

impl JwtValidator for ReadOnly {
    fn validate(_claims: &serde_json::Value) -> Result<Self, String> {
        Ok(ReadOnly {})
    }
}

/// Validates that a token grants read and write access
pub struct ReadWrite {}

impl JwtValidator for ReadWrite {
    fn validate(claims: &serde_json::Value) -> Result<Self, String> {
        if let Some(flag) = claims["ptet:write"].as_bool() {
            if flag {
                Ok(ReadWrite {})
            } else {
                Err("ptet:write claim is false".to_string())
            }
        } else {
            Err("No ptet:write claim in JWT".to_string())
        }
    }
}
