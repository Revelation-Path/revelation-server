use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post, put}
};
use masterror::prelude::*;
use revelation_shared::{Church, CreateChurch, JoinChurch, Membership, UpdateChurch, UpdateMemberRole};
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

async fn create_church(
    State(state): State<AppState>,
    Json(payload): Json<CreateChurchRequest>
) -> AppResult<Json<Church>> {
    let id = Uuid::now_v7();

    let church = sqlx::query_as!(
        Church,
        r#"
        INSERT INTO churches (id, name, city, address, confession_id, admin_id, latitude, longitude)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, name, city, address, confession_id, admin_id, latitude, longitude, created_at
        "#,
        id,
        payload.church.name,
        payload.church.city,
        payload.church.address,
        payload.church.confession_id,
        payload.admin_id,
        payload.church.latitude,
        payload.church.longitude
    )
    .fetch_one(&state.pool)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO memberships (id, user_id, church_id, role)
        VALUES ($1, $2, $3, 'admin')
        "#,
        Uuid::now_v7(),
        payload.admin_id,
        id
    )
    .execute(&state.pool)
    .await?;

    Ok(Json(church))
}

#[derive(serde::Deserialize)]
pub struct CreateChurchRequest {
    admin_id: Uuid,
    #[serde(flatten)]
    church:   CreateChurch
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

async fn update_church(
    State(state): State<AppState>,
    Path(church_id): Path<Uuid>,
    Json(payload): Json<UpdateChurch>
) -> AppResult<Json<Church>> {
    let church = sqlx::query_as!(
        Church,
        r#"
        UPDATE churches SET
            name = COALESCE($2, name),
            address = COALESCE($3, address),
            latitude = COALESCE($4, latitude),
            longitude = COALESCE($5, longitude)
        WHERE id = $1
        RETURNING id, name, city, address, confession_id, admin_id, latitude, longitude, created_at
        "#,
        church_id,
        payload.name,
        payload.address,
        payload.latitude,
        payload.longitude
    )
    .fetch_one(&state.pool)
    .await?;

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
    role:      revelation_shared::ChurchRole,
    joined_at: chrono::DateTime<chrono::Utc>
}

async fn join_church(
    State(state): State<AppState>,
    Path(church_id): Path<Uuid>,
    Json(payload): Json<JoinChurchRequest>
) -> AppResult<Json<Membership>> {
    let id = Uuid::now_v7();

    let membership = sqlx::query_as!(
        Membership,
        r#"
        INSERT INTO memberships (id, user_id, church_id, role)
        VALUES ($1, $2, $3, $4)
        RETURNING id, user_id, church_id, role as "role: _", joined_at
        "#,
        id,
        payload.user_id,
        church_id,
        payload.join.role as _
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(membership))
}

#[derive(serde::Deserialize)]
pub struct JoinChurchRequest {
    user_id: Uuid,
    #[serde(flatten)]
    join:    JoinChurch
}

async fn update_member_role(
    State(state): State<AppState>,
    Path((church_id, user_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateMemberRole>
) -> AppResult<Json<Membership>> {
    let membership = sqlx::query_as!(
        Membership,
        r#"
        UPDATE memberships SET role = $3
        WHERE church_id = $1 AND user_id = $2
        RETURNING id, user_id, church_id, role as "role: _", joined_at
        "#,
        church_id,
        user_id,
        payload.role as _
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(membership))
}
