/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;
use std::sync::RwLock;
use rocket::fairing::AdHoc;
use chrono::{DateTime, TimeDelta, Utc};

/// Rocket state for authentication cache
pub struct AuthCache {
    /// Key cache
    pub key_cache: RwLock<jwt_auth::keys::KeyCache>,
    /// Expected audience in JWT
    pub expect_jwt_audience: String,
    /// Expected issuer in JWT
    pub expect_jwt_issuer: Option<String>,
    /// JWT must be issued later than.
    /// Can be used to ban tempered token before a certain issuing time. But it has
    /// the side effect that it invalidates all tokens before this time.
    pub jwt_issued_after: Option<DateTime<Utc>>,
    /// Maximum expiration time
    pub jwt_max_expiration: TimeDelta,
}

/// Fairing for key cache
pub fn init(
    key_cache_path: PathBuf,
    expect_jwt_audience: String,
    expect_jwt_issuer: Option<String>,
    jwt_issued_after: Option<DateTime<Utc>>,
    jwt_max_expiration: TimeDelta,
) -> AdHoc {
    AdHoc::on_ignite(
        "Initializing key cache",
        move |rocket| async move {
            let key_cache = jwt_auth::keys::KeyCache::from_path(key_cache_path).unwrap();
            let state = AuthCache {
                key_cache: RwLock::new(key_cache),
                expect_jwt_audience,
                expect_jwt_issuer,
                jwt_issued_after,
                jwt_max_expiration,
            };
            rocket.manage(state)
        }
    )
}
