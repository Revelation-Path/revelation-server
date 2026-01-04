// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status:  &'static str,
    version: &'static str
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status:  "ok",
        version: env!("CARGO_PKG_VERSION")
    })
}
