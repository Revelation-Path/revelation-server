use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put}
};
use masterror::prelude::*;
use revelation_shared::{BindEmail, BindPhone, BindTelegram, CreateUser, UpdateUserProfile, User};
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_user))
        .route("/{user_id}", get(get_user))
        .route("/{user_id}/profile", put(update_profile))
        .route("/{user_id}/bind/telegram", post(bind_telegram))
        .route("/{user_id}/bind/email", post(bind_email))
        .route("/{user_id}/bind/phone", post(bind_phone))
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>
) -> AppResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
        r#"
        INSERT INTO users (id)
        VALUES ($1)
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
        payload.id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user))
}

async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>
) -> AppResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
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
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user))
}

async fn update_profile(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateUserProfile>
) -> AppResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
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
        user_id,
        payload.name,
        payload.gender as _,
        payload.birth_date,
        payload.confession_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user))
}

async fn bind_telegram(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<BindTelegram>
) -> AppResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
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
        user_id,
        payload.telegram_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user))
}

async fn bind_email(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<BindEmail>
) -> AppResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
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
        user_id,
        payload.email
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user))
}

async fn bind_phone(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<BindPhone>
) -> AppResult<Json<User>> {
    let user = sqlx::query_as!(
        User,
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
        user_id,
        payload.phone
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(user))
}
