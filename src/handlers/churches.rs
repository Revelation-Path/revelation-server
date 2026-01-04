// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put}
};
use masterror::prelude::*;
use revelation_church::{
    Church, ChurchRole, CreateChurch, JoinChurch, Membership, UpdateChurch, UpdateMemberRole
};
use revelation_user::Claims;
use uuid::Uuid;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(create_church))
        .route("/{church_id}", get(get_church))
        .route("/{church_id}", put(update_church))
        .route("/{church_id}/members", get(get_members))
        .route("/{church_id}/join", post(join_church))
        .route(
            "/{church_id}/members/{user_id}/role",
            put(update_member_role)
        )
}

/// Create a new church. The authenticated user becomes the admin.
async fn create_church(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateChurch>
) -> AppResult<Json<Church>> {
    let id = Uuid::now_v7();
    let admin_id = claims.user_id();

    let mut tx = state.pool.begin().await?;

    let church = sqlx::query_as!(
        Church,
        r#"
        INSERT INTO churches (id, name, city, address, confession_id, admin_id, latitude, longitude)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, city, address, confession_id, admin_id, latitude, longitude, created_at
        "#,
        id,
        payload.name,
        payload.city,
        payload.address,
        payload.confession_id,
        admin_id,
        payload.latitude,
        payload.longitude
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO memberships (id, user_id, church_id, role)
        VALUES ($1, $2, $3, 'admin')
        "#,
        Uuid::now_v7(),
        admin_id,
        id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(church))
}

async fn get_church(
    State(state): State<AppState>,
    Path(church_id): Path<Uuid>
) -> AppResult<Json<Church>> {
    let church = sqlx::query_as!(
        Church,
        r#"
        SELECT id, name, city, address, confession_id, admin_id, latitude, longitude, created_at
        FROM churches
        WHERE id = $1
        "#,
        church_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(church))
}

/// Update church. Only admin can update.
async fn update_church(
    State(state): State<AppState>,
    claims: Claims,
    Path(church_id): Path<Uuid>,
    Json(payload): Json<UpdateChurch>
) -> AppResult<Json<Church>> {
    let user_id = claims.user_id();

    let church = sqlx::query_as!(
        Church,
        r#"
        UPDATE churches SET
            name = COALESCE($2, name),
            address = COALESCE($3, address),
            latitude = COALESCE($4, latitude),
            longitude = COALESCE($5, longitude)
        WHERE id = $1 AND admin_id = $6
        RETURNING id, name, city, address, confession_id, admin_id, latitude, longitude, created_at
        "#,
        church_id,
        payload.name,
        payload.address,
        payload.latitude,
        payload.longitude,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::forbidden("Not authorized to update this church"))?;

    Ok(Json(church))
}

async fn get_members(
    State(state): State<AppState>,
    Path(church_id): Path<Uuid>
) -> AppResult<Json<Vec<MemberWithUser>>> {
    let members = sqlx::query_as!(
        MemberWithUser,
        r#"
        SELECT
            m.id,
            m.user_id,
            u.name as user_name,
            m.church_id,
            m.role as "role: _",
            m.joined_at
        FROM memberships m
        JOIN users u ON u.id = m.user_id
        WHERE m.church_id = $1
        ORDER BY m.joined_at
        "#,
        church_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(members))
}

#[derive(serde::Serialize)]
pub struct MemberWithUser {
    id:        Uuid,
    user_id:   Uuid,
    user_name: Option<String>,
    church_id: Uuid,
    role:      ChurchRole,
    joined_at: chrono::DateTime<chrono::Utc>
}

/// Join a church. User joins as the role specified (default: guest).
async fn join_church(
    State(state): State<AppState>,
    claims: Claims,
    Path(church_id): Path<Uuid>,
    Json(payload): Json<JoinChurch>
) -> AppResult<Json<Membership>> {
    let id = Uuid::now_v7();
    let user_id = claims.user_id();

    let membership = sqlx::query_as!(
        Membership,
        r#"
        INSERT INTO memberships (id, user_id, church_id, role)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, church_id, role as "role: _", joined_at
        "#,
        id,
        user_id,
        church_id,
        payload.role as _
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(membership))
}

/// Update member role. Only admin or pastor can change roles.
async fn update_member_role(
    State(state): State<AppState>,
    claims: Claims,
    Path((church_id, target_user_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMemberRole>
) -> AppResult<Json<Membership>> {
    let user_id = claims.user_id();

    let caller_role = sqlx::query_scalar!(
        r#"SELECT role as "role: ChurchRole" FROM memberships WHERE church_id = $1 AND user_id = $2"#,
        church_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| AppError::forbidden("You are not a member of this church"))?;

    if !matches!(caller_role, ChurchRole::Admin | ChurchRole::Pastor) {
        return Err(AppError::forbidden("Only admin or pastor can change roles"));
    }

    let membership = sqlx::query_as!(
        Membership,
        r#"
        UPDATE memberships SET role = $3
        WHERE church_id = $1 AND user_id = $2
        RETURNING id, user_id, church_id, role as "role: _", joined_at
        "#,
        church_id,
        target_user_id,
        payload.role as _
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(membership))
}
