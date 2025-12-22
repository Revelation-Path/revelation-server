use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::get
};
use masterror::prelude::*;
use serde::Deserialize;
use revelation_shared::{Book, ChapterInfo, DailyReading, Pericope, SearchResult, Testament, Verse};

use crate::state::AppState;

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

#[derive(Deserialize)]
pub struct BooksQuery {
    testament: Option<Testament>
}

async fn get_books(
    State(state): State<AppState>,
    Query(query): Query<BooksQuery>
) -> AppResult<Json<Vec<Book>>> {
    let books = match query.testament {
        Some(testament) => {
            state
                .bible
                .repository()
                .get_books_by_testament(testament)
                .await?
        }
        None => state.bible.repository().get_books().await?
    };
    Ok(Json(books))
}

async fn get_chapter(
    State(state): State<AppState>,
    Path((book_id, chapter)): Path<(i16, i16)>
) -> AppResult<Json<Vec<Verse>>> {
    let verses = state
        .bible
        .repository()
        .get_chapter(book_id, chapter)
        .await?;
    Ok(Json(verses))
}

async fn get_pericopes(
    State(state): State<AppState>,
    Path(book_id): Path<i16>
) -> AppResult<Json<Vec<Pericope>>> {
    let pericopes = state.bible.repository().get_pericopes(book_id).await?;
    Ok(Json(pericopes))
}

async fn get_chapters_info(
    State(state): State<AppState>,
    Path(book_id): Path<i16>
) -> AppResult<Json<Vec<ChapterInfo>>> {
    let info = state.bible.repository().get_chapters_info(book_id).await?;
    Ok(Json(info))
}

async fn get_verse(
    State(state): State<AppState>,
    Path((book_id, chapter, verse)): Path<(i16, i16, i16)>
) -> AppResult<Json<Option<Verse>>> {
    let verse = state
        .bible
        .repository()
        .get_verse(book_id, chapter, verse)
        .await?;
    Ok(Json(verse))
}

#[derive(Deserialize)]
pub struct SearchQuery {
    q:     String,
    #[serde(default = "default_limit")]
    limit: i64
}

fn default_limit() -> i64 {
    50
}

async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>
) -> AppResult<Json<Vec<SearchResult>>> {
    let results = state.bible.search().search(&query.q, query.limit).await?;
    Ok(Json(results))
}

async fn symphony(
    State(state): State<AppState>,
    Path(word): Path<String>,
    Query(query): Query<LimitQuery>
) -> AppResult<Json<SymphonyResponse>> {
    let count = state.bible.search().word_count(&word).await?;
    let verses = state
        .bible
        .search()
        .symphony(&word, query.limit.unwrap_or(100))
        .await?;

    Ok(Json(SymphonyResponse {
        word,
        total_count: count,
        verses
    }))
}

#[derive(Deserialize)]
pub struct LimitQuery {
    limit: Option<i64>
}

#[derive(serde::Serialize)]
pub struct SymphonyResponse {
    word:        String,
    total_count: i64,
    verses:      Vec<SearchResult>
}

async fn get_today_reading(
    State(state): State<AppState>
) -> AppResult<Json<Option<DailyReading>>> {
    let reading = state.bible.reading_plan().get_today().await?;
    Ok(Json(reading))
}

async fn get_day_reading(
    State(state): State<AppState>,
    Path(day): Path<i16>
) -> AppResult<Json<Option<DailyReading>>> {
    let reading = state.bible.reading_plan().get_for_day(day).await?;
    Ok(Json(reading))
}
