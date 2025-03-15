/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod fairings;
mod request_guards;
mod routes;

use std::error::Error;
use std::path::PathBuf;
use chrono::{DateTime, TimeDelta, Utc};
use clap::Parser;
use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};

#[macro_use] extern crate rocket;

/// CLI interface
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to the keys
    #[arg(short, long, default_value = "8080")]
    port: u16,
    /// Database URI for SeaORM
    #[arg(short, long)]
    database: String,
    /// Path to the key cache
    #[arg(short, long)]
    keys_dir: PathBuf,
    /// Server base URI
    #[arg(short = 'u', long)]
    server_base_uri: String,
    /// Optionally, restrict accepted JWTs to issuer
    #[arg(long)]
    expect_jwt_issuer: Option<String>,
    /// Optionally, only accept issued after a certain time
    #[arg(long)]
    jwt_issued_after: Option<DateTime<Utc>>,
    /// Set maximum expiration time
    #[arg(long, default_value = "31536000")]
    jwt_max_expiration: i64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    rocket::build()
        .configure(
            rocket::Config{
                port: cli.port,
                ..rocket::Config::default()
            }
        )
        .attach(
            fairings::auth_cache::init(
                cli.keys_dir.clone(),
                cli.server_base_uri.clone(),
                cli.expect_jwt_issuer.clone(),
                cli.jwt_issued_after,
                TimeDelta::seconds(cli.jwt_max_expiration),
            )
        )
        .mount(
            "/api/v1/",
            openapi_get_routes![
                routes::user::get,
            ]
        )
        .mount(
            "/api/v1/docs/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/v1/openapi.json".to_owned(),
                ..SwaggerUIConfig::default()
            })
        )
        .launch()
        .await?;

    Ok(())
}
