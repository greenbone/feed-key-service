// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::Router;

use crate::app::AppRouter;

pub mod health;
pub mod key;

pub fn routes(upload_limit: Option<usize>) -> AppRouter {
    Router::new()
        .nest("/key", key::routes(upload_limit))
        .nest("/health", health::routes())
}
