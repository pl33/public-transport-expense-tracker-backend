/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use rand::{distr::Alphanumeric, Rng};
use openssl::pkey::{PKey, Private, Public};
use super::key_store::KeyStore;
use super::key_generator::KeyGenerator;

/// In-memory cache for keys
pub struct KeyCache {
    key_store: KeyStore,
    private_keys: HashMap<String, PKey<Private>>,
    public_keys: HashMap<String, PKey<Public>>,
    default_key_id: Option<String>,
}

impl KeyCache {
    const DEFAULT_KEY_ID_LEN: usize = 16;
    const DEFAULT_RSA_BITS: u32 = 2048;

    /// New key cache from path
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let store = KeyStore::new(path);
        Self::new(store)
    }

    /// Create a new key cache
    pub fn new(key_store: KeyStore) -> Result<Self, Box<dyn Error>> {
        // Read default key ID or use last key ID in list
        let default_key_id = match key_store.default_key_id()? {
            Some(key_id) => { Some(key_id) },
            None => {
                let mut key_ids = key_store.key_id_list()?;
                match key_ids.pop() {
                    Some(key_id) => {
                        key_store.make_default(key_id.as_str())?;
                        Some(key_id)
                    },
                    None => None,
                }
            }
        };
        Ok(
            Self {
                key_store,
                private_keys: HashMap::new(),
                public_keys: HashMap::new(),
                default_key_id,
            }
        )
    }
}

impl<'a> KeyCache {
    /// Get private key with ID [key_id]
    pub fn create_private_key(&'a mut self, key_id: Option<&str>, generator: Option<KeyGenerator>) -> Result<(&'a PKey<Private>, String), Box<dyn Error>> {
        // Create a random key ID if none was given
        let key_id = match key_id {
            Some(key_id) => String::from(key_id),
            None => {
                let key_id: String = rand::rng()
                    .sample_iter(&Alphanumeric)
                    .take(Self::DEFAULT_KEY_ID_LEN)
                    .map(char::from)
                    .collect();
                key_id
            },
        };

        // Use RSA 2048 by default
        let generator = generator.unwrap_or_else(|| KeyGenerator::Rsa { bits: Self::DEFAULT_RSA_BITS });

        let private_key = self.key_store.create_key_pair(
            key_id.as_str(),
            generator,
        )?;

        // If this is the first key, make it the default one
        if let None = self.default_key_id {
            self.key_store.make_default(key_id.as_str())?;
            self.default_key_id = Some(key_id.clone());
        }

        self.private_keys.insert(key_id.clone(), private_key);
        Ok((&self.private_keys[key_id.as_str()], key_id))
    }

    /// If [key_id] is Some, return it. If it is None, return [default_key_id]. If
    /// [default_key_id] is None, too, return with an error.
    fn default_key_if_none(key_id: Option<&'a str>, default_key_id: &'a Option<String>) -> Result<&'a str, Box<dyn Error>> {
        match key_id {
            Some(key_id) => Ok(key_id),
            None => {
                match default_key_id {
                    Some(key_id) => Ok(key_id.as_str()),
                    None => Err(From::from("key_id is None and no default key could be obtained")),
                }
            }
        }
    }

    /// Get private key with ID [key_id], or the default private key if [key_id] is None
    pub fn get_private_key(&'a mut self, key_id: Option<&str>) -> Result<(&'a PKey<Private>, String), Box<dyn Error>> {
        let key_id = Self::default_key_if_none(key_id, &self.default_key_id)?;

        if !self.private_keys.contains_key(key_id) {
            self.private_keys.insert(String::from(key_id), self.key_store.load_private_key(key_id)?);
        }
        Ok((&self.private_keys[key_id], key_id.to_string()))
    }

    /// Get public key with ID [key_id]
    pub fn get_public_key(&'a mut self, key_id: Option<&str>) -> Result<(&'a PKey<Public>, String), Box<dyn Error>> {
        let key_id = Self::default_key_if_none(key_id, &self.default_key_id)?;

        if !self.public_keys.contains_key(key_id) {
            self.public_keys.insert(String::from(key_id), self.key_store.load_public_key(key_id)?);
        }
        Ok((&self.public_keys[key_id], key_id.to_string()))
    }

    /// List all key IDs
    pub fn key_id_list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        self.key_store.key_id_list()
    }
}
