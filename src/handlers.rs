// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use axum::{Router, routing::get};
use utoipa::{
    Modify, OpenApi,
    openapi::{
        OpenApi as UtoipaOpenApi,
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme}
    }
};

use crate::state::AppState;

mod bible;
mod churches;
mod feed;
mod health;
mod songs;
mod users;

pub use bible::BibleApiDoc;
pub use songs::SongsApiDoc;
pub use users::UsersApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(),
    components(),
    modifiers(&SecurityAddon),
    info(
        title = "Revelation API",
        version = env!("CARGO_PKG_VERSION"),
        description = "Revelation Platform API - Bible, Songbook, and User Management"
    ),
    servers(
        (url = "https://api.revelation-path.ru", description = "Production"),
        (url = "http://localhost:3000", description = "Local development")
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Users", description = "User management (requires auth)"),
        (name = "Bible", description = "Bible reading endpoints (public)"),
        (name = "Songs", description = "Songbook endpoints (public read, auth for write)"),
        (name = "Churches", description = "Church endpoints"),
        (name = "Feed", description = "Feed endpoints")
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut UtoipaOpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "cookieAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build()
                )
            );
        }
    }
}

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .nest("/bible", bible::routes())
        .nest("/songs", songs::routes())
        .nest("/users", users::routes())
        .nest("/churches", churches::routes())
        .nest("/feed", feed::routes())
}

pub fn merged_openapi() -> UtoipaOpenApi {
    let mut openapi = ApiDoc::openapi();
    openapi.merge(BibleApiDoc::openapi());
    openapi.merge(SongsApiDoc::openapi());
    openapi.merge(UsersApiDoc::openapi());
    openapi
}
