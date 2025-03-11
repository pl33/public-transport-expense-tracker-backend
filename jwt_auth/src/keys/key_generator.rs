/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use openssl::rsa::Rsa;
use openssl::nid::Nid;
use openssl::ec::{EcKey, EcGroup};
use openssl::pkey::{PKey, Private};

/// Key generators
pub enum KeyGenerator {
    Rsa { bits: u32 },
    Ec { group: EcGroup },
}

impl KeyGenerator {
    /// Generator with creates an RSA key
    pub fn new_rsa(bits: u32) -> Self {
        KeyGenerator::Rsa { bits }
    }

    /// Generator with creates an Ecliptic Curve key
    pub fn new_ec(group: EcGroup) -> Self {
        KeyGenerator::Ec { group }
    }

    /// Generator with creates an Ecliptic Curve key from NID curve
    pub fn new_ec_from_nid(nid: Nid) -> Result<Self, Box<dyn Error>> {
        let group = EcGroup::from_curve_name(nid)?;
        Ok(KeyGenerator::Ec { group })
    }

    /// Generate private key with configured parameters
    pub fn generate(self) -> Result<PKey<Private>, Box<dyn Error>> {
        let key = match self {
            Self::Rsa { bits } => {
                let key = Rsa::generate(bits)?;
                PKey::from_rsa(key)?
            },
            Self::Ec { group } => {
                let key = EcKey::generate(&group)?;
                PKey::from_ec_key(key)?
            },
        };
        Ok(key)
    }
}

#[cfg(test)]
mod test {
    use openssl::nid::Nid;
    use crate::keys::key_generator::KeyGenerator;

    #[test]
    fn test_generate_rsa() {
        let gen = KeyGenerator::new_rsa(2048);
        let key = gen.generate().unwrap();
        assert_eq!(key.bits(), 2048);
    }

    #[test]
    fn test_generate_ec() {
        let gen = KeyGenerator::new_ec_from_nid(Nid::X9_62_PRIME256V1).unwrap();
        let key = gen.generate().unwrap();
        assert_eq!(key.bits(), 256);
    }
}
