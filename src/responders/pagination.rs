/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2025 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use rocket::{Request, Response};
use rocket::http::{ContentType, Header, Status};
use rocket::response::Responder;
use rocket_okapi::gen::OpenApiGenerator;
use rocket_okapi::okapi::openapi3::Responses;
use rocket_okapi::response::OpenApiResponderInner;

pub enum PaginatedResult<R> {
    Paginated {
        result: R,
        item_count: u64,
        page: u64,
        page_size: u64,
        pages_count: u64,
    },
    Complete {
        result: R,
        item_count: Option<u64>,
    },
}

impl<'r, 'o: 'r, R: Responder<'r, 'o>> PaginatedResult<R> {
    pub fn new_paginated(result: R, item_count: u64, page: u64, page_size: u64) -> Self {
        let pages_count = (item_count / page_size) + if item_count % page_size != 0 { 1 } else { 0 };
        Self::Paginated {
            result,
            item_count,
            page,
            page_size,
            pages_count,
        }
    }

    pub fn new_complete(result: R, item_count: Option<u64>) -> Self {
        Self::Complete {
            result,
            item_count,
        }
    }
}

impl<'r, 'o: 'r, R: Responder<'r, 'o>> Responder<'r, 'o> for PaginatedResult<R> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'o> {
        match self {
            PaginatedResult::Paginated {
                result,
                item_count,
                page,
                page_size,
                pages_count,
            } => {
                let uri = request.uri().path();
                let mut links = format!("<{uri}?page={page}&size={page_size}>; rel=\"self\"");
                links += format!(", <{uri}?page=0&size={page_size}>; rel=\"first\"").as_str();
                let last_page = pages_count - 1;
                links += format!(", <{uri}?page={last_page}&size={page_size}>; rel=\"last\"").as_str();
                if page > 0 {
                    let prev_page = if page < last_page {
                        page - 1
                    } else {
                        last_page
                    };
                    links += format!(", <{uri}?page={prev_page}&size={page_size}>; rel=\"prev\"").as_str();
                }
                if page < last_page {
                    let next_page = page + 1;
                    links += format!(", <{uri}?page={next_page}&size={page_size}>; rel=\"next\"").as_str();
                }
                Response::build_from(result.respond_to(request)?)
                    .status(Status::Ok)
                    .header(ContentType::JSON)
                    .header(Header::new("X-Total-Items", format!("{item_count}")))
                    .header(Header::new("X-Page", format!("{page}")))
                    .header(Header::new("X-Page-Size", format!("{page_size}")))
                    .header(Header::new("X-Total-pages", format!("{pages_count}")))
                    .header(Header::new("Link", links))
                    .ok()
            },
            PaginatedResult::Complete {
                result,
                item_count,
            } => {
                let mut res = Response::build_from(result.respond_to(request)?);
                res.status(Status::Ok);
                res.header(ContentType::JSON);
                if let Some(item_count) = item_count {
                    res.header(Header::new("X-Total-Items", format!("{item_count}")));
                }
                res.ok()
            },
        }
    }
}

impl<R> OpenApiResponderInner for PaginatedResult<R> {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        Ok(Responses {
            responses: rocket_okapi::okapi::map! {
                // Note: 401 is already declared for ApiKey. so this is not essential.
                // "401".to_owned() => RefOr::Object(unauthorized_response(gen)),
            },
            ..Default::default()
        })
    }
}
