/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fs;
use std::path::{Path, PathBuf};
use std::error::Error;
use openssl::pkey::{PKey, Public, Private};
use super::key_generator::KeyGenerator;

/// Facade to keys
///
/// All keys are stored at [base_dir]/key_[key_id]/{public,private}.pem
pub struct KeyStore {
    /// Base directory where the keys are stored
    base_dir: PathBuf,
}

impl KeyStore {
    const KEY_DIR_PREFIX: &'static str = "key_";
    const DEFAULT_TXT: &'static str = "default.txt";
    const PUBLIC_PEM: &'static str = "public.pem";
    const PRIVATE_PEM: &'static str = "private.pem";

    /// Create a new key store with [base_dir] as base directory
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Path to directory of key with ID [key_id]
    fn key_dir(&self, key_id: &str) -> PathBuf {
        let mut key_path = self.base_dir.clone();
        let dir_name = String::from(Self::KEY_DIR_PREFIX) + key_id;
        key_path.push(dir_name);
        key_path
    }

    /// Create key pair with ID [key_id]
    pub fn create_key_pair(&self, key_id: &str, generator: KeyGenerator) -> Result<PKey<Private>, Box<dyn Error>> {
        let key_path = self.key_dir(key_id);

        if key_path.exists() {
            Err(From::from("Key already exists"))
        } else {
            fs::create_dir_all(&key_path)?;

            let private_key = generator.generate()?;

            {
                let mut private_key_path = key_path.clone();
                private_key_path.push(Self::PRIVATE_PEM);
                fs::write(&private_key_path, private_key.private_key_to_pem_pkcs8()?)?;
            }

            {
                let mut public_key_path = key_path.clone();
                public_key_path.push(Self::PUBLIC_PEM);
                let public_pem = private_key.public_key_to_pem()?;
                fs::write(&public_key_path, public_pem.as_slice())?;
            }

            Ok(private_key)
        }
    }

    /// Load public key with ID [key_id]
    pub fn load_public_key(&self, key_id: &str) -> Result<PKey<Public>, Box<dyn Error>> {
        let mut public_key_path = self.key_dir(key_id);
        public_key_path.push(Self::PUBLIC_PEM);

        if public_key_path.is_file() {
            let pem_str = fs::read_to_string(public_key_path)?;
            let key = PKey::public_key_from_pem(pem_str.as_bytes())?;
            Ok(key)
        } else {
            Err(From::from("Public key file not found"))
        }
    }

    /// Load private key with ID [key_id]
    pub fn load_private_key(&self, key_id: &str) -> Result<PKey<Private>, Box<dyn Error>> {
        let mut private_key_path = self.key_dir(key_id);
        private_key_path.push(Self::PRIVATE_PEM);

        if private_key_path.is_file() {
            let pem_str = fs::read_to_string(private_key_path)?;
            let key = PKey::private_key_from_pem(pem_str.as_bytes())?;
            Ok(key)
        } else {
            Err(From::from("Private key file not found"))
        }
    }

    /// Get list of keys
    pub fn key_id_list(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let mut key_ids = Vec::new();
        for dir in fs::read_dir(&self.base_dir)? {
            let dir = dir?;
            let dir_name = dir.file_name().to_str().unwrap().to_owned();
            if dir.file_type()?.is_dir() && dir_name.starts_with(Self::KEY_DIR_PREFIX) {
                let key_id = &dir_name[Self::KEY_DIR_PREFIX.len()..];
                key_ids.push(String::from(key_id));
            }
        }
        Ok(key_ids)
    }

    /// Set [key_id] as default
    pub fn make_default(&self, key_id: &str) -> Result<(), Box<dyn Error>> {
        let mut default_txt_path = self.base_dir.clone();
        default_txt_path.push(Self::DEFAULT_TXT);
        fs::write(&default_txt_path, key_id.as_bytes())?;
        Ok(())
    }

    /// Get default key ID
    pub fn default_key_id(&self) -> Result<Option<String>, Box<dyn Error>> {
        let mut default_txt_path = self.base_dir.clone();
        default_txt_path.push(Self::DEFAULT_TXT);
        if default_txt_path.is_file() {
            let key_id = String::from_utf8(fs::read(&default_txt_path)?)?;
            Ok(Some(key_id))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use openssl::nid::Nid;
    use tempfile::TempDir;
    use crate::keys::key_generator::KeyGenerator;
    use crate::keys::key_store::KeyStore;

    #[test]
    fn test_key_store() {
        let tmp_dir = TempDir::new().unwrap();
        let key_store = KeyStore::new(tmp_dir.path());

        assert_eq!(key_store.default_key_id().unwrap(), None);

        let test1_private = key_store.create_key_pair(
            "test1",
            KeyGenerator::new_rsa(2048),
        ).unwrap();
        let test1_public = key_store.load_public_key("test1").unwrap();
        assert!(test1_private.public_eq(&test1_public));

        let test2_private = key_store.create_key_pair(
            "test2",
            KeyGenerator::new_ec_from_nid(Nid::X9_62_PRIME256V1).unwrap(),
        ).unwrap();
        let test2_public = key_store.load_public_key("test2").unwrap();
        assert!(test2_private.public_eq(&test2_public));

        let key_id_list = key_store.key_id_list().unwrap();
        assert_eq!(key_id_list.len(), 2);
        assert!(key_id_list.contains(&String::from("test1")));
        assert!(key_id_list.contains(&String::from("test2")));

        key_store.make_default("test1").unwrap();
        assert_eq!(key_store.default_key_id().unwrap(), Some(String::from("test1")));
    }
}
