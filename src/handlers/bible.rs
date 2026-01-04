// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! Bible API handlers

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get
};
use masterror::prelude::*;
use revelation_bible::{
    Book, ChapterInfo, DailyReading, Pericope, SearchResult, Testament, Verse
};
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(paths(
    get_books,
    get_chapter,
    get_pericopes,
    get_chapters_info,
    get_verse,
    search,
    symphony,
    get_today_reading,
    get_day_reading
))]
pub struct BibleApiDoc;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/books", get(get_books))
        .route("/books/{book_id}/chapters/{chapter}", get(get_chapter))
        .route("/books/{book_id}/pericopes", get(get_pericopes))
        .route("/books/{book_id}/chapters-info", get(get_chapters_info))
        .route(
            "/books/{book_id}/chapters/{chapter}/verses/{verse}",
            get(get_verse)
        )
        .route("/search", get(search))
        .route("/symphony/{word}", get(symphony))
        .route("/today", get(get_today_reading))
        .route("/day/{day}", get(get_day_reading))
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/books",
    params(
        ("testament" = Option<String>, Query, description = "Filter by testament: old or new")
    ),
    responses(
        (status = 200, description = "List of books", body = Vec<Book>)
    )
)]
async fn get_books(
    State(state): State<AppState>,
    Query(query): Query<BooksQueryInternal>
) -> AppResult<Json<Vec<Book>>> {
    let books = match query.testament {
        Some(testament) => state.bible.get_books_by_testament(testament).await?,
        None => state.bible.get_books().await?
    };
    Ok(Json(books))
}

#[derive(Deserialize)]
struct BooksQueryInternal {
    testament: Option<Testament>
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/books/{book_id}/chapters/{chapter}",
    params(
        ("book_id" = i16, Path, description = "Book ID"),
        ("chapter" = i16, Path, description = "Chapter number")
    ),
    responses(
        (status = 200, description = "Chapter verses", body = Vec<Verse>),
        (status = 404, description = "Book or chapter not found")
    )
)]
async fn get_chapter(
    State(state): State<AppState>,
    Path((book_id, chapter)): Path<(i16, i16)>
) -> AppResult<Json<Vec<Verse>>> {
    let verses = state.bible.get_chapter(book_id, chapter).await?;
    Ok(Json(verses))
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/books/{book_id}/pericopes",
    params(
        ("book_id" = i16, Path, description = "Book ID")
    ),
    responses(
        (status = 200, description = "Book pericopes", body = Vec<Pericope>)
    )
)]
async fn get_pericopes(
    State(state): State<AppState>,
    Path(book_id): Path<i16>
) -> AppResult<Json<Vec<Pericope>>> {
    let pericopes = state.bible.get_pericopes(book_id).await?;
    Ok(Json(pericopes))
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/books/{book_id}/chapters-info",
    params(
        ("book_id" = i16, Path, description = "Book ID")
    ),
    responses(
        (status = 200, description = "Chapters info", body = Vec<ChapterInfo>)
    )
)]
async fn get_chapters_info(
    State(state): State<AppState>,
    Path(book_id): Path<i16>
) -> AppResult<Json<Vec<ChapterInfo>>> {
    let info = state.bible.get_chapters_info(book_id).await?;
    Ok(Json(info))
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/books/{book_id}/chapters/{chapter}/verses/{verse}",
    params(
        ("book_id" = i16, Path, description = "Book ID"),
        ("chapter" = i16, Path, description = "Chapter number"),
        ("verse" = i16, Path, description = "Verse number")
    ),
    responses(
        (status = 200, description = "Single verse", body = Option<Verse>)
    )
)]
async fn get_verse(
    State(state): State<AppState>,
    Path((book_id, chapter, verse)): Path<(i16, i16, i16)>
) -> AppResult<Json<Option<Verse>>> {
    let verse = state.bible.get_verse(book_id, chapter, verse).await?;
    Ok(Json(verse))
}

#[derive(Deserialize, ToSchema)]
pub struct SearchQuery {
    q:     String,
    #[serde(default = "default_limit")]
    limit: i64
}

fn default_limit() -> i64 {
    50
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<i64>, Query, description = "Max results (default 50)")
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<SearchResult>)
    )
)]
async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>
) -> AppResult<Json<Vec<SearchResult>>> {
    let results = state.bible.search(&query.q, query.limit).await?;
    Ok(Json(results))
}

#[derive(Deserialize, ToSchema)]
pub struct LimitQuery {
    limit: Option<i64>
}

#[derive(serde::Serialize, ToSchema)]
pub struct SymphonyResponse {
    word:        String,
    total_count: i64
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/symphony/{word}",
    params(
        ("word" = String, Path, description = "Word to search"),
        ("limit" = Option<i64>, Query, description = "Max results (default 100)")
    ),
    responses(
        (status = 200, description = "Word occurrences", body = SymphonyResponse)
    )
)]
async fn symphony(
    State(state): State<AppState>,
    Path(word): Path<String>,
    Query(query): Query<LimitQuery>
) -> AppResult<Json<SymphonyResponseFull>> {
    let count = state.bible.word_count(&word).await?;
    let verses = state
        .bible
        .symphony(&word, query.limit.unwrap_or(100))
        .await?;

    Ok(Json(SymphonyResponseFull {
        word,
        total_count: count,
        verses
    }))
}

#[derive(serde::Serialize)]
struct SymphonyResponseFull {
    word:        String,
    total_count: i64,
    verses:      Vec<SearchResult>
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/today",
    responses(
        (status = 200, description = "Today's reading", body = Option<DailyReading>)
    )
)]
async fn get_today_reading(
    State(state): State<AppState>
) -> AppResult<Json<Option<DailyReading>>> {
    let reading = state.bible.get_today().await?;
    Ok(Json(reading))
}

#[utoipa::path(
    get,
    tag = "Bible",
    path = "/api/bible/day/{day}",
    params(
        ("day" = i16, Path, description = "Day of year (1-366)")
    ),
    responses(
        (status = 200, description = "Reading for day", body = Option<DailyReading>)
    )
)]
async fn get_day_reading(
    State(state): State<AppState>,
    Path(day): Path<i16>
) -> AppResult<Json<Option<DailyReading>>> {
    let reading = state.bible.get_for_day(day).await?;
    Ok(Json(reading))
}
