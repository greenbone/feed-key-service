// SPDX-FileCopyrightText: 2026 Greenbone AG
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use axum::Router;

use crate::app::AppRouter;

pub mod health;
pub mod key;

pub fn routes() -> AppRouter {
    Router::new()
        .nest("/key", key::routes())
        .nest("/health", health::routes())
}
