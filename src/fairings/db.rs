/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::sync::Arc;
use rocket::fairing::AdHoc;

/// Database state in Rocket
pub struct Database {
    /// Database connection
    pub conn: Arc<sea_orm::DatabaseConnection>,
}

/// Fairing for database setup
pub fn init(url: String) -> AdHoc {
    AdHoc::on_ignite(
        "Connecting to database",
        move |rocket| async {
            let conn = sea_orm::Database::connect(url).await.unwrap();
            let db = Database {
                conn: Arc::new(conn),
            };

            use migration::{Migrator, MigratorTrait};
            Migrator::up(db.conn.as_ref(), None).await.unwrap();

            rocket.manage(db)
        }
    )
}
