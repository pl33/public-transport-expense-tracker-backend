/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::BTreeMap;
use std::error::Error;
use openssl::hash::MessageDigest;
use jwt::{Token, Header, algorithm::openssl::PKeyWithDigest, SigningAlgorithm, Claims, RegisteredClaims, SignWithKey, token::Signed};
use chrono::{DateTime, Utc};
use rand::distr::Alphanumeric;
use rand::Rng;
use crate::keys::KeyCache;

/// Producer for JWT
pub struct TokenProducer<'cache, 'kid> {
    key_cache: &'cache mut KeyCache,
    key_id: Option<&'kid str>,
    issuer: Option<String>,
    not_before: Option<DateTime<Utc>>,
    expiration: Option<DateTime<Utc>>,
    audience: Option<String>,
    token_id: Option<String>,
    additional_claims: BTreeMap<String, serde_json::Value>,
    now: DateTime<Utc>,
}

impl<'cache, 'kid> TokenProducer<'cache, 'kid> {
    const DEFAULT_WEB_TOKEN_ID_LENGTH: usize = 20;
    
    pub fn new(key_cache: &'cache mut KeyCache) -> Self {
        Self { 
            key_cache,
            key_id: None,
            issuer: None,
            not_before: None,
            expiration: None,
            audience: None,
            token_id: None,
            additional_claims: BTreeMap::new(),
            now: Utc::now(),
        }
    }
    
    /// Set key ID
    pub fn with_key_id(mut self, key_id: &'kid str) -> Self {
        self.key_id = Some(key_id);
        self
    }

    /// Set issuer
    pub fn with_issuer<S: ToString>(mut self, issuer: S) -> Self {
        self.issuer = Some(issuer.to_string());
        self
    }

    /// Set not before field
    pub fn with_not_before(mut self, not_before: DateTime<Utc>) -> Self {
        self.not_before = Some(not_before);
        self
    }

    /// Set expiration field
    pub fn with_expiration(mut self, expiration: DateTime<Utc>) -> Self {
        self.expiration = Some(expiration);
        self
    }

    /// Set audience
    pub fn with_audience<S: ToString>(mut self, audience: S) -> Self {
        self.audience = Some(audience.to_string());
        self
    }

    /// Set web token ID
    pub fn with_token_id<S: ToString>(mut self, token_id: S) -> Self {
        self.token_id = Some(token_id.to_string());
        self
    }

    /// Set random web token ID
    pub fn with_random_token_id(mut self, length: Option<usize>) -> Self {
        let token_id = rand::rng()
            .sample_iter(&Alphanumeric)
            .take(length.unwrap_or(Self::DEFAULT_WEB_TOKEN_ID_LENGTH))
            .map(char::from)
            .collect();
        self.token_id = Some(token_id);
        self
    }

    /// Add additional claim, string value
    pub fn add_claim_string<K: ToString, V: ToString>(mut self, claim: K, value: V) -> Self {
        self.additional_claims.insert(
            claim.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        self
    }

    /// Add additional claim from JSON value
    pub fn add_claims_from_json(mut self, value: serde_json::Value) -> Result<Self, Box<dyn Error>> {
        if let serde_json::Value::Object(obj) = value {
            for (k, v) in obj {
                self.additional_claims.insert(k, v);
            }
            Ok(self)
        } else {
            Err("Expected JSON object")?
        }
    }

    /// Produces a new token
    pub fn produce(self, subject: &str) -> Result<Token<Header, Claims, Signed>, Box<dyn Error>> {
        let (key, key_id) = self.key_cache.get_private_key(self.key_id)?;
        let alg = PKeyWithDigest {
            key: key.clone(),
            digest: MessageDigest::sha512(),
        };

        let header = Header {
            algorithm: alg.algorithm_type(),
            key_id: Some(key_id),
            ..Default::default()
        };

        let mut claims = Claims::new(
            RegisteredClaims {
                issuer: self.issuer,
                subject: Some(subject.to_string()),
                audience: self.audience,
                issued_at: Some(self.now.timestamp() as u64),
                not_before: self.not_before.map(|e| e.timestamp() as u64),
                expiration: self.expiration.map(|e| e.timestamp() as u64),
                json_web_token_id: self.token_id,
            },
        );
        claims.private = self.additional_claims;
        let token = Token::new(header, claims);
        Ok(token.sign_with_key(&alg)?)
    }
}
