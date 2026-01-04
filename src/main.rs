// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! Revelation Server - REST API for the Revelation ecosystem.
//!
//! This is the main entry point for the server binary.

use axum::{Extension, Router};
use masterror::prelude::*;
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod handlers;
mod middleware;
mod state;

use config::Config;
use middleware::create_auth_extensions;
use state::AppState;

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenvy::dotenv().ok();
    init_tracing();

    let config = Config::from_env();
    let pool = create_pool(&config).await?;
    let state = AppState::new(pool);
    let app = create_app(state, &config);

    let addr = config.socket_addr();
    tracing::info!("Server listening on {addr}");
    tracing::info!(
        "Swagger UI: http://{}:{}/swagger-ui",
        config.host,
        config.port
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize tracing subscriber with env filter.
fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Create database connection pool.
async fn create_pool(config: &Config) -> AppResult<sqlx::PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await?;

    Ok(pool)
}

/// Create the application router with all middleware.
fn create_app(state: AppState, config: &Config) -> Router {
    let (jwt_validator, auth_config) =
        create_auth_extensions(&config.jwt_secret, &config.cookie_name);

    let cors = create_cors_layer(config);

    Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .nest("/api", handlers::api_routes())
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", handlers::merged_openapi())
        )
        .layer(Extension(jwt_validator))
        .layer(Extension(auth_config))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
}

/// Create CORS layer based on configuration.
fn create_cors_layer(config: &Config) -> CorsLayer {
    if config.is_cors_permissive() {
        tracing::warn!("ALLOWED_ORIGINS not set, using permissive CORS (development only!)");
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(
                config
                    .allowed_origins
                    .iter()
                    .filter_map(|o| o.parse().ok())
                    .collect::<Vec<_>>()
            )
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_credentials(true)
    }
}
