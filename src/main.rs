// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use std::net::SocketAddr;

use axum::{Extension, Router};
use masterror::prelude::*;
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa_swagger_ui::SwaggerUi;

mod handlers;
mod middleware;
mod state;

use middleware::create_auth_extensions;
use state::AppState;

/// Server configuration from environment variables.
struct Config {
    database_url: String,
    jwt_secret: String,
    cookie_name: String,
    host: String,
    port: u16,
    max_connections: u32,
    allowed_origins: Vec<String>,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// # Panics
    ///
    /// Panics if required environment variables are not set.
    fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            cookie_name: std::env::var("COOKIE_NAME").unwrap_or_else(|_| "auth_token".into()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3000),
            max_connections: std::env::var("MAX_CONNECTIONS")
                .ok()
                .and_then(|c| c.parse().ok())
                .unwrap_or(10),
            allowed_origins: std::env::var("ALLOWED_ORIGINS")
                .map(|s| s.split(',').map(String::from).collect())
                .unwrap_or_default(),
        }
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.database_url)
        .await?;

    let state = AppState::new(pool);

    let (jwt_validator, auth_config) =
        create_auth_extensions(&config.jwt_secret, &config.cookie_name);

    let cors = if config.allowed_origins.is_empty() {
        tracing::warn!("ALLOWED_ORIGINS not set, using permissive CORS (development only!)");
        CorsLayer::permissive()
    } else {
        CorsLayer::new()
            .allow_origin(
                config
                    .allowed_origins
                    .iter()
                    .filter_map(|o| o.parse().ok())
                    .collect::<Vec<_>>(),
            )
            .allow_methods(Any)
            .allow_headers(Any)
            .allow_credentials(true)
    };

    let app = Router::new()
        .route("/health", axum::routing::get(|| async { "ok" }))
        .nest("/api", handlers::api_routes())
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", handlers::merged_openapi()),
        )
        .layer(Extension(jwt_validator))
        .layer(Extension(auth_config))
        .with_state(state)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid HOST:PORT");

    tracing::info!("Server listening on {}", addr);
    tracing::info!("Swagger UI: http://{}:{}/swagger-ui", config.host, config.port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
