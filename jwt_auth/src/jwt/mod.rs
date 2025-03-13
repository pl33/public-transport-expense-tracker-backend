/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

pub mod token_producer;
mod token_verifier;

pub use token_producer::TokenProducer;
pub use token_verifier::TokenVerifier;


#[cfg(test)]
mod tests {
    use openssl::nid::Nid;
    use tempfile::TempDir;
    use crate::jwt::{TokenProducer, TokenVerifier};
    use crate::keys::key_generator::KeyGenerator;
    use crate::keys::KeyCache;

    #[test]
    fn test_token_produce_verify() {

        let tmp_dir = TempDir::new().unwrap();
        let mut key_cache = KeyCache::from_path(tmp_dir.path()).unwrap();

        key_cache.create_private_key(
            Some("test1"),
            Some(KeyGenerator::new_rsa(2048)),
        ).unwrap();
        key_cache.create_private_key(
            Some("test2"),
            Some(KeyGenerator::new_ec_from_nid(Nid::SECP521R1).unwrap()),
        ).unwrap();

        let token_produced = TokenProducer::new(&mut key_cache)
            .with_issuer("issuer@example.tld")
            .with_key_id("test1")
            .with_audience("resource.example.tld")
            .with_token_id("qwertyuiop")
            .produce("subject@example.tld")
            .unwrap();
        let token_str = String::from(token_produced);
        let (token_decoded, key_id) = TokenVerifier::new(&mut key_cache)
            .disable_time_check()
            .verify(token_str)
            .unwrap();

        assert_eq!(key_id, "test1");
        assert_eq!(token_decoded.claims().registered.subject, Some("subject@example.tld".to_string()));
        assert_eq!(token_decoded.claims().registered.issuer, Some("issuer@example.tld".to_string()));
        assert_eq!(token_decoded.claims().registered.audience, Some("resource.example.tld".to_string()));
        assert_eq!(token_decoded.claims().registered.json_web_token_id, Some("qwertyuiop".to_string()));
    }
}

