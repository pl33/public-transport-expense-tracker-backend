/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::path::PathBuf;
use chrono::{DateTime, Utc, TimeDelta};
use clap::{Parser, Subcommand};
use jwt_auth::keys::KeyCache;
use jwt_auth::keys::KeyGenerator;
use jwt_auth::jwt::TokenProducer;
use jwt_auth::jwt::TokenVerifier;

/// Create tokens
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the keys
    #[arg(short, long, default_value = "./keys/")]
    key_dir: PathBuf,

    /// Command to execute
    #[command(subcommand)]
    action: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new key
    CreateKey {
        /// Key ID
        #[arg(short, long)]
        key_id: Option<String>,
    },
    /// List keys
    ListKeys,
    /// Show public key
    ShowPublic {
        /// Key ID
        key_id: String,
    },
    /// Create a new token
    CreateToken {
        /// Key ID
        #[arg(short, long)]
        key_id: Option<String>,
        /// Issuer
        #[arg(short, long)]
        issuer: Option<String>,
        /// Audience
        #[arg(short, long)]
        audience: Option<String>,
        /// Not before date time string
        #[arg(short, long)]
        not_before: Option<DateTime<Utc>>,
        /// Expiration date time string
        #[arg(short, long)]
        expiration: Option<DateTime<Utc>>,
        /// Addition claims, in the form: key=value
        #[arg(short, long)]
        claim: Vec<String>,
        /// Addition claims, as JSON object
        #[arg(long)]
        claims_json: Option<String>,
        /// Subject
        subject: String,
    },
    /// Verify token
    VerifyToken {
        /// Key ID
        #[arg(short = 'k', long)]
        expect_key_id: Option<String>,
        /// Issuer
        #[arg(short = 'i', long)]
        expect_issuer: Option<String>,
        /// Audience
        #[arg(short = 'a', long)]
        expect_audience: Option<String>,
        /// Maximum expiration from issuing time in seconds
        #[arg(short = 'e', long)]
        max_expiration: Option<i64>,
        /// Token
        token: String,
    }
}

fn main() {
    let cli = Cli::parse();
    
    let mut key_cache = KeyCache::from_path(&cli.key_dir).unwrap();
    
    match cli.action {
        Commands::CreateKey { key_id } => {
            let (key, key_id) = key_cache.create_private_key(
                match &key_id { 
                    Some(id) => Some(id.as_str()),
                    None => None,
                },
                Some(KeyGenerator::new_rsa(2048)),
            ).unwrap();
            
            println!("Key ID: {}", key_id);
            println!("Public Key:\n{}", String::from_utf8(key.public_key_to_pem().unwrap()).unwrap());
        },
        Commands::ListKeys => {
            for key_id in key_cache.key_id_list().unwrap() {
                println!("{}", key_id);
            }
        },
        Commands::ShowPublic { key_id } => {
            let (key, _) = key_cache.get_public_key(Some(key_id.as_str())).unwrap();
            println!("{}", String::from_utf8(key.public_key_to_pem().unwrap()).unwrap());
        },
        Commands::CreateToken {
            key_id,
            issuer,
            audience,
            not_before,
            expiration,
            claim,
            claims_json,
            subject,
        } => {
            let mut token_producer = TokenProducer::new(&mut key_cache);
            if let Some(key_id) = &key_id {
                token_producer = token_producer.with_key_id(key_id.as_str());
            }
            if let Some(issuer) = issuer {
                token_producer = token_producer.with_issuer(issuer);
            }
            if let Some(audience) = audience {
                token_producer = token_producer.with_audience(audience);
            }
            if let Some(not_before) = not_before {
                token_producer = token_producer.with_not_before(not_before);
            }
            if let Some(expiration) = expiration {
                token_producer = token_producer.with_expiration(expiration);
            }
            for item in &claim {
                let mut iter = item.split('=');
                let key = match iter.next() {
                    Some(value) => value,
                    None => panic!("Cannot parse claim, missing key"),
                };
                let value = match iter.next() {
                    Some(value) => value,
                    None => panic!("Cannot parse claim, missing value"),
                };
                match iter.next() {
                    Some(_value) => panic!("Cannot parse claim, too many ="),
                    None => (),
                };
                token_producer = token_producer.add_claim_string(key, value);
            }
            if let Some(claims) = claims_json {
                let value: serde_json::Value = serde_json::from_str(&claims).unwrap();
                token_producer = token_producer.add_claims_from_json(value).unwrap();
            }
            let token = token_producer.produce(&subject).unwrap();
            println!("{}", String::from(token))
        },
        Commands::VerifyToken {
            expect_key_id,
            expect_issuer,
            expect_audience,
            max_expiration,
            token,
        } => {
            let mut verifier = TokenVerifier::new(&mut key_cache);
            if let Some(key_id) = &expect_key_id {
                verifier = verifier.expect_key_id(key_id.as_str());
            }
            if let Some(issuer) = &expect_issuer {
                verifier = verifier.expect_issuer(issuer);
            }
            if let Some(audience) = &expect_audience {
                verifier = verifier.expect_audience(audience);
            }
            if let Some(max_expiration) = max_expiration {
                verifier = verifier.with_max_expiration(TimeDelta::seconds(max_expiration));
            }
            let (token, key_id) = verifier.verify(token).unwrap();
            println!("Token was signed with key: {}", key_id);
            if let Some(subject) = &token.claims().registered.subject {
                println!("Token subject is: {}", subject);
            }
            if let Some(token_id) = &token.claims().registered.json_web_token_id {
                println!("Token Web Token ID is: {}", token_id);
            }
        }
    }
}