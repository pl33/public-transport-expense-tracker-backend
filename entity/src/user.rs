/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, DeriveEntityModel, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename = "User")]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: u32,
    #[serde(skip_deserializing)]
    pub jwt_issuer: String,
    #[serde(skip_deserializing)]
    pub jwt_subject: String,
    pub name: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::ride::Entity")]
    Ride,
}

impl Related<super::ride::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Ride.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
