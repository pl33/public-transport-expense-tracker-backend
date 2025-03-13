/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */
use std::error::Error;
use chrono::{DateTime, Utc, TimeDelta};
use jwt::{Claims, Header, PKeyWithDigest, Token, Unverified, Verified, VerifyWithKey};
use openssl::hash::MessageDigest;
use crate::keys::KeyCache;

/// Verifier for JWT
pub struct TokenVerifier<'cache, 'kid> {
    key_cache: &'cache mut KeyCache,
    key_id: Option<&'kid str>,
    issuer: Option<String>,
    audience: Option<String>,
    check_times: bool,
    max_expiration: Option<TimeDelta>,
    issued_after: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
}

impl<'cache, 'kid> TokenVerifier<'cache, 'kid> {
    pub fn new(key_cache: &'cache mut KeyCache) -> Self {
        Self {
            key_cache,
            key_id: None,
            issuer: None,
            audience: None,
            check_times: true,
            max_expiration: None,
            issued_after: None,
            now: Utc::now(),
        }
    }

    /// Set expected key ID
    pub fn expect_key_id(mut self, key_id: &'kid str) -> Self {
        self.key_id = Some(key_id);
        self
    }

    /// Set expected issuer
    pub fn expect_issuer<S: ToString>(mut self, issuer: S) -> Self {
        self.issuer = Some(issuer.to_string());
        self
    }

    /// Set expected audience
    pub fn expect_audience<S: ToString>(mut self, audience: S) -> Self {
        self.audience = Some(audience.to_string());
        self
    }

    /// Disable check if validity time
    pub fn disable_time_check(mut self) -> Self {
        self.check_times = false;
        self
    }

    /// Restrict expiration time to a delta from now
    pub fn with_max_expiration(mut self, max_expiration_from_now: TimeDelta) -> Self {
        self.max_expiration = Some(max_expiration_from_now);
        self
    }

    pub fn must_be_issued_after(mut self, issued_after: DateTime<Utc>) -> Self {
        self.issued_after = Some(issued_after);
        self
    }

    /// Verify token and return key ID used to sign the token
    pub fn verify<S: AsRef<str>>(self, token: S) -> Result<(Token<Header, Claims, Verified>, String), Box<dyn Error>> {
        let token: Token<Header, Claims, Unverified> = Token::parse_unverified(token.as_ref())?;
        let key_id = match &token.header().key_id {
            Some(key_id) => Some(key_id.as_str()),
            None => None,
        };

        let (key, key_id) = self.key_cache.get_public_key(key_id)?;
        let alg = PKeyWithDigest {
            key: key.clone(),
            digest: MessageDigest::sha512(),
        };

        // Check key ID
        if let Some(expected_key_id) = self.key_id {
            if expected_key_id != key_id {
                Err("Key ID does not match")?;
            }
        }

        // Verify token signature and decode it
        let token: Token<Header, Claims, Verified> = token.verify_with_key(&alg)?;

        // Check issuer
        if let Some(expected_issuer) = self.issuer {
            match &token.claims().registered.issuer {
                Some(issuer) => {
                    if expected_issuer.ne(issuer) {
                        Err("Issuer does not match")?;
                    }
                },
                None => Err("Issuer not set in token")?,
            }
        }

        // Check audience
        if let Some(expected_audience) = self.audience {
            match &token.claims().registered.audience {
                Some(audience) => {
                    if expected_audience.ne(audience) {
                        Err("Audience does not match")?;
                    }
                },
                None => Err("Audience not set in token")?,
            }
        }

        // Check issue time
        if let Some(issued_after) = self.issued_after {
            match token.claims().registered.issued_at {
                Some(issued_at) => {
                    if issued_at < (issued_after.timestamp() as u64) {
                        Err("Audience does not match")?;
                    }
                },
                None => Err("Issued at not set in token")?,
            }
        }

        // Check validity time
        if self.check_times {
            match token.claims().registered.not_before {
                Some(not_before) => {
                    if not_before > (self.now.timestamp() as u64) {
                        Err("Token is not valid yet")?;
                    }
                },
                None => (),
            }
            let issued_at = match token.claims().registered.issued_at {
                Some(issued_at) => issued_at,
                None => return Err("Issued at not set in token")?,
            };
            match token.claims().registered.expiration {
                Some(expiration) => {
                    if let Some(max_expiration) = self.max_expiration {
                        if expiration > (issued_at + (max_expiration.num_seconds() as u64)) {
                            Err("Token expiration time exceeds maximum allowed value")?;
                        }
                    }
                    if expiration < (self.now.timestamp() as u64) {
                        Err("Token is expired")?;
                    }
                },
                None => Err("Token has no expiration time")?,
            }
        }

        Ok((token, key_id))
    }
}
