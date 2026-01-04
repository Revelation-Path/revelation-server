// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use axum::{
    Json, Router,
    extract::State,
    routing::{get, post, put}
};
use masterror::prelude::*;
use revelation_user::{
    BindEmail, BindPhone, BindTelegram, Claims, RUser, RUserPublic, UpdateProfileRequest
};
use utoipa::OpenApi;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(paths(get_me, update_profile, bind_telegram, bind_email, bind_phone))]
pub struct UsersApiDoc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(get_me))
        .route("/me/profile", put(update_profile))
        .route("/me/bind/telegram", post(bind_telegram))
        .route("/me/bind/email", post(bind_email))
        .route("/me/bind/phone", post(bind_phone))
}

#[utoipa::path(
    get,
    tag = "Users",
    path = "/api/users/me",
    responses(
        (status = 200, description = "Current user profile", body = RUserPublic),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookieAuth" = []))
)]
async fn get_me(State(state): State<AppState>, claims: Claims) -> AppResult<Json<RUserPublic>> {
    let user = sqlx::query_as!(
        RUser,
        r#"
        SELECT
            id,
            name,
            gender as "gender: _",
            birth_date,
            confession_id,
            email,
            phone,
            telegram_id,
            created_at,
            updated_at
        FROM users
        WHERE id = $1
        "#,
        claims.user_id()
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    put,
    tag = "Users",
    path = "/api/users/me/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = RUserPublic),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Validation error")
    ),
    security(("cookieAuth" = []))
)]
async fn update_profile(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateProfileRequest>
) -> AppResult<Json<RUserPublic>> {
    let user = sqlx::query_as!(
        RUser,
        r#"
        UPDATE users SET
            name = COALESCE($2, name),
            gender = COALESCE($3, gender),
            birth_date = COALESCE($4, birth_date),
            confession_id = COALESCE($5, confession_id),
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            name,
            gender as "gender: _",
            birth_date,
            confession_id,
            email,
            phone,
            telegram_id,
            created_at,
            updated_at
        "#,
        claims.user_id(),
        payload.name,
        payload.gender as _,
        payload.birth_date,
        payload.confession_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    post,
    tag = "Users",
    path = "/api/users/me/bind/telegram",
    request_body = BindTelegram,
    responses(
        (status = 200, description = "Telegram bound", body = RUserPublic),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookieAuth" = []))
)]
async fn bind_telegram(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<BindTelegram>
) -> AppResult<Json<RUserPublic>> {
    let user = sqlx::query_as!(
        RUser,
        r#"
        UPDATE users SET telegram_id = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            name,
            gender as "gender: _",
            birth_date,
            confession_id,
            email,
            phone,
            telegram_id,
            created_at,
            updated_at
        "#,
        claims.user_id(),
        payload.telegram_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    post,
    tag = "Users",
    path = "/api/users/me/bind/email",
    request_body = BindEmail,
    responses(
        (status = 200, description = "Email bound", body = RUserPublic),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookieAuth" = []))
)]
async fn bind_email(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<BindEmail>
) -> AppResult<Json<RUserPublic>> {
    let user = sqlx::query_as!(
        RUser,
        r#"
        UPDATE users SET email = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            name,
            gender as "gender: _",
            birth_date,
            confession_id,
            email,
            phone,
            telegram_id,
            created_at,
            updated_at
        "#,
        claims.user_id(),
        payload.email
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user.into()))
}

#[utoipa::path(
    post,
    tag = "Users",
    path = "/api/users/me/bind/phone",
    request_body = BindPhone,
    responses(
        (status = 200, description = "Phone bound", body = RUserPublic),
        (status = 401, description = "Unauthorized")
    ),
    security(("cookieAuth" = []))
)]
async fn bind_phone(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<BindPhone>
) -> AppResult<Json<RUserPublic>> {
    let user = sqlx::query_as!(
        RUser,
        r#"
        UPDATE users SET phone = $2, updated_at = NOW()
        WHERE id = $1
        RETURNING
            id,
            name,
            gender as "gender: _",
            birth_date,
            confession_id,
            email,
            phone,
            telegram_id,
            created_at,
            updated_at
        "#,
        claims.user_id(),
        payload.phone
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user.into()))
}
