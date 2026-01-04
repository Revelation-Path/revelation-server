// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post}
};
use masterror::prelude::*;
use revelation_post::{CreateComment, CreatePost, Post, PostComment};
use revelation_user::Claims;
use uuid::Uuid;

use crate::state::AppState;

/// Maximum posts per request.
const MAX_LIMIT: i64 = 100;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_feed))
        .route("/", post(create_post))
        .route("/{post_id}", get(get_post))
        .route("/{post_id}/comments", get(get_comments))
        .route("/{post_id}/comments", post(create_comment))
}

#[derive(serde::Deserialize)]
pub struct FeedQuery {
    #[serde(default)]
    church_id: Option<Uuid>,
    #[serde(default = "default_limit")]
    limit:     i64,
    #[serde(default)]
    offset:    i64
}

const fn default_limit() -> i64 {
    20
}

async fn get_feed(
    State(state): State<AppState>,
    Query(query): Query<FeedQuery>
) -> AppResult<Json<Vec<PostWithAuthor>>> {
    let limit = query.limit.min(MAX_LIMIT);

    let posts = sqlx::query_as!(
        PostWithAuthor,
        r#"
        SELECT
            p.id,
            p.author_id,
            u.name as author_name,
            p.church_id,
            c.name as church_name,
            p.post_type as "post_type: _",
            p.title,
            p.content,
            p.media_urls,
            p.created_at,
            p.updated_at
        FROM posts p
        JOIN users u ON u.id = p.author_id
        LEFT JOIN churches c ON c.id = p.church_id
        WHERE ($1::uuid IS NULL AND p.church_id IS NULL) OR p.church_id = $1
        ORDER BY p.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
        query.church_id,
        limit,
        query.offset
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(posts))
}

#[derive(serde::Serialize)]
pub struct PostWithAuthor {
    id:          Uuid,
    author_id:   Uuid,
    author_name: Option<String>,
    church_id:   Option<Uuid>,
    church_name: Option<String>,
    post_type:   revelation_post::PostType,
    title:       String,
    content:     String,
    media_urls:  Vec<String>,
    created_at:  chrono::DateTime<chrono::Utc>,
    updated_at:  chrono::DateTime<chrono::Utc>
}

async fn get_post(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>
) -> AppResult<Json<Post>> {
    let post = sqlx::query_as!(
        Post,
        r#"
        SELECT
            id,
            author_id,
            church_id,
            post_type as "post_type: _",
            title,
            content,
            media_urls,
            created_at,
            updated_at
        FROM posts
        WHERE id = $1
        "#,
        post_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(post))
}

/// Create a post. Author is taken from JWT claims.
async fn create_post(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreatePost>
) -> AppResult<Json<Post>> {
    let id = Uuid::now_v7();
    let author_id = claims.user_id();

    let post = sqlx::query_as!(
        Post,
        r#"
        INSERT INTO posts (id, author_id, church_id, post_type, title, content, media_urls)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id,
            author_id,
            church_id,
            post_type as "post_type: _",
            title,
            content,
            media_urls,
            created_at,
            updated_at
        "#,
        id,
        author_id,
        payload.church_id,
        payload.post_type as _,
        payload.title,
        payload.content,
        &payload.media_urls
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(post))
}

async fn get_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>
) -> AppResult<Json<Vec<CommentWithAuthor>>> {
    let comments = sqlx::query_as!(
        CommentWithAuthor,
        r#"
        SELECT
            c.id,
            c.post_id,
            c.author_id,
            u.name as author_name,
            c.content,
            c.created_at
        FROM post_comments c
        JOIN users u ON u.id = c.author_id
        WHERE c.post_id = $1
        ORDER BY c.created_at ASC
        "#,
        post_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(comments))
}

#[derive(serde::Serialize)]
pub struct CommentWithAuthor {
    id:          Uuid,
    post_id:     Uuid,
    author_id:   Uuid,
    author_name: Option<String>,
    content:     String,
    created_at:  chrono::DateTime<chrono::Utc>
}

/// Create a comment. Author is taken from JWT claims.
async fn create_comment(
    State(state): State<AppState>,
    claims: Claims,
    Path(post_id): Path<Uuid>,
    Json(payload): Json<CreateComment>
) -> AppResult<Json<PostComment>> {
    let id = Uuid::now_v7();
    let author_id = claims.user_id();

    let comment = sqlx::query_as!(
        PostComment,
        r#"
        INSERT INTO post_comments (id, post_id, author_id, content)
        VALUES ($1, $2, $3, $4)
        RETURNING id, post_id, author_id, content, created_at
        "#,
        id,
        post_id,
        author_id,
        payload.content
    )
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(comment))
}
