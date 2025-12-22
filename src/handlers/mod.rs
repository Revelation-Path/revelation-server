use axum::{Router, routing::get};

use crate::state::AppState;

mod bible;
mod churches;
mod feed;
mod health;
mod songs;
mod users;

pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health::health_check))
        .nest("/bible", bible::routes())
        .nest("/songs", songs::routes())
        .nest("/users", users::routes())
        .nest("/churches", churches::routes())
        .nest("/feed", feed::routes())
}
