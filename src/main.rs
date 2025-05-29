/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

mod fairings;
mod request_guards;
mod model;
mod responders;
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
        .attach(fairings::db::init(cli.database.clone()))
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
                routes::user::put,
                routes::ride::list,
                routes::ride::post,
                routes::ride::get,
                routes::ride::put,
                routes::ride::delete,
                routes::ride_tag::list,
                routes::ride_tag::get_by_tag_id,
                routes::ride_tag::post_by_tag_id,
                routes::ride_tag::get_by_link_id,
                routes::ride_tag::put,
                routes::ride_tag::delete,
                routes::tag::list,
                routes::tag::post,
                routes::tag::get,
                routes::tag::put,
                routes::tag::delete,
                routes::tag_option::list,
                routes::tag_option::post,
                routes::tag_option::get,
                routes::tag_option::put,
                routes::tag_option::delete,
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
