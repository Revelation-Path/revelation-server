//! Song API handlers

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{delete, get, post}
};
use masterror::prelude::*;
use revelation_songbook::{
    AddToPlaylist, CreatePlaylist, CreateSong, PlaylistItem, Song, SongCategory, SongFilters,
    SongHistoryEntry, SongPlaylist, SongSearchResult, SongSortBy, SongSummary, SongTag, Songbook,
    SongbookEdition, UpdateSong, transpose_content, transpose_key
};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;

/// Song routes
pub fn routes() -> Router<AppState> {
    Router::new()
        // Songbooks
        .route("/songbooks", get(list_songbooks))
        .route("/songbooks/{id}", get(get_songbook))
        .route("/songbooks/{id}/editions", get(get_songbook_editions))
        .route("/songbooks/{id}/songs", get(list_songbook_songs))
        // Songs
        .route("/", get(list_songs).post(create_song))
        .route("/search", get(search_songs))
        .route("/categories", get(list_categories))
        .route("/categories/{category}", get(list_by_category))
        .route("/tags", get(list_tags))
        .route("/{id}", get(get_song).put(update_song).delete(delete_song))
        .route("/{id}/transpose/{semitones}", get(get_song_transposed))
        // Favorites
        .route("/favorites", get(list_favorites))
        .route(
            "/favorites/{song_id}",
            post(add_favorite).delete(remove_favorite)
        )
        // History
        .route("/history", get(list_history))
        // Playlists
        .route("/playlists", get(list_playlists).post(create_playlist))
        .route("/playlists/{id}", get(get_playlist).delete(delete_playlist))
        .route(
            "/playlists/{id}/songs",
            get(get_playlist_songs).post(add_to_playlist)
        )
        .route(
            "/playlists/{id}/songs/{item_id}",
            delete(remove_from_playlist)
        )
}

// ============================================================================
// Songbooks
// ============================================================================

/// List all public songbooks
async fn list_songbooks(State(state): State<AppState>) -> AppResult<Json<Vec<Songbook>>> {
    let songbooks = state.songs.list_songbooks().await?;
    Ok(Json(songbooks))
}

/// Get songbook by ID
async fn get_songbook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
) -> AppResult<Json<Songbook>> {
    let songbook = state.songs.get_songbook(id).await?;
    Ok(Json(songbook))
}

/// Get songbook editions
async fn get_songbook_editions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
) -> AppResult<Json<Vec<SongbookEdition>>> {
    let editions = state.songs.get_songbook_editions(id).await?;
    Ok(Json(editions))
}

/// List songs in a songbook
async fn list_songbook_songs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<SongListQuery>
) -> AppResult<Json<Vec<SongSummary>>> {
    let filters = SongFilters {
        songbook_id: Some(id),
        limit: query.limit,
        offset: query.offset,
        sort_by: Some(SongSortBy::Number),
        ..Default::default()
    };

    let songs = state.songs.list_songs(&filters, query.user_id).await?;
    Ok(Json(songs))
}

// ============================================================================
// Songs
// ============================================================================

#[derive(Debug, Deserialize)]
struct SongListQuery {
    songbook_id: Option<Uuid>,
    category:    Option<SongCategory>,
    tag_id:      Option<Uuid>,
    key:         Option<String>,
    search:      Option<String>,
    #[serde(default)]
    limit:       Option<i64>,
    #[serde(default)]
    offset:      Option<i64>,
    sort_by:     Option<SongSortBy>,
    user_id:     Option<Uuid>
}

/// List songs with filters
async fn list_songs(
    State(state): State<AppState>,
    Query(query): Query<SongListQuery>
) -> AppResult<Json<Vec<SongSummary>>> {
    let filters = SongFilters {
        songbook_id: query.songbook_id,
        category:    query.category,
        tag_id:      query.tag_id,
        key:         query.key,
        search:      query.search,
        limit:       query.limit,
        offset:      query.offset,
        sort_by:     query.sort_by
    };

    let songs = state.songs.list_songs(&filters, query.user_id).await?;
    Ok(Json(songs))
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q:       String,
    #[serde(default = "default_limit")]
    limit:   i64,
    user_id: Option<Uuid>
}

fn default_limit() -> i64 {
    50
}

/// Search songs by text
async fn search_songs(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>
) -> AppResult<Json<Vec<SongSearchResult>>> {
    let results = state
        .songs
        .search_songs(&query.q, query.limit, query.user_id)
        .await?;
    Ok(Json(results))
}

/// Get song by ID
async fn get_song(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<UserQuery>
) -> AppResult<Json<Song>> {
    let song = state.songs.get_song(id, query.user_id).await?;
    Ok(Json(song))
}

#[derive(Debug, Deserialize)]
struct UserQuery {
    user_id: Option<Uuid>
}

/// Get song transposed by semitones
async fn get_song_transposed(
    State(state): State<AppState>,
    Path((id, semitones)): Path<(Uuid, i32)>,
    Query(query): Query<UserQuery>
) -> AppResult<Json<Song>> {
    let mut song = state.songs.get_song(id, query.user_id).await?;

    song.content = transpose_content(&song.content, semitones);

    if let Some(key) = &song.original_key {
        song.original_key = Some(transpose_key(key, semitones, false));
    }

    Ok(Json(song))
}

/// Create a new song
async fn create_song(
    State(state): State<AppState>,
    Json(song): Json<CreateSong>
) -> AppResult<Json<Song>> {
    let created = state.songs.create_song(song).await?;
    Ok(Json(created))
}

/// Update an existing song
async fn update_song(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(song): Json<UpdateSong>
) -> AppResult<Json<Song>> {
    let updated = state.songs.update_song(id, song).await?;
    Ok(Json(updated))
}

/// Delete a song
async fn delete_song(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<()> {
    state.songs.delete_song(id).await?;
    Ok(())
}

// ============================================================================
// Categories & Tags
// ============================================================================

/// List all categories with Russian names
async fn list_categories() -> Json<Vec<CategoryInfo>> {
    let categories = SongCategory::all()
        .iter()
        .map(|c| CategoryInfo {
            category: *c,
            name_ru:  c.name_ru().to_string()
        })
        .collect();

    Json(categories)
}

#[derive(serde::Serialize)]
struct CategoryInfo {
    category: SongCategory,
    name_ru:  String
}

#[derive(Debug, Deserialize)]
struct CategoryQuery {
    #[serde(default = "default_limit")]
    limit:   i64,
    user_id: Option<Uuid>
}

/// List songs by category
async fn list_by_category(
    State(state): State<AppState>,
    Path(category): Path<SongCategory>,
    Query(query): Query<CategoryQuery>
) -> AppResult<Json<Vec<SongSummary>>> {
    let songs = state
        .songs
        .list_by_category(category, query.limit, query.user_id)
        .await?;
    Ok(Json(songs))
}

/// List all tags
async fn list_tags(State(state): State<AppState>) -> AppResult<Json<Vec<SongTag>>> {
    let tags = state.songs.list_tags().await?;
    Ok(Json(tags))
}

// ============================================================================
// Favorites
// ============================================================================

#[derive(Debug, Deserialize)]
struct FavoritesQuery {
    user_id: Uuid
}

/// List user's favorite songs
async fn list_favorites(
    State(state): State<AppState>,
    Query(query): Query<FavoritesQuery>
) -> AppResult<Json<Vec<SongSummary>>> {
    let songs = state.songs.list_favorites(query.user_id).await?;
    Ok(Json(songs))
}

#[derive(Debug, Deserialize)]
struct AddFavoriteQuery {
    user_id: Uuid
}

/// Add song to favorites
async fn add_favorite(
    State(state): State<AppState>,
    Path(song_id): Path<Uuid>,
    Query(query): Query<AddFavoriteQuery>
) -> AppResult<()> {
    state.songs.add_favorite(query.user_id, song_id).await?;
    Ok(())
}

/// Remove song from favorites
async fn remove_favorite(
    State(state): State<AppState>,
    Path(song_id): Path<Uuid>,
    Query(query): Query<AddFavoriteQuery>
) -> AppResult<()> {
    state.songs.remove_favorite(query.user_id, song_id).await?;
    Ok(())
}

// ============================================================================
// History
// ============================================================================

#[derive(Debug, Deserialize)]
struct HistoryQuery {
    user_id: Uuid,
    #[serde(default = "default_history_limit")]
    limit:   i64
}

fn default_history_limit() -> i64 {
    20
}

/// List user's recent songs
async fn list_history(
    State(state): State<AppState>,
    Query(query): Query<HistoryQuery>
) -> AppResult<Json<Vec<SongHistoryEntry>>> {
    let history = state.songs.list_recent(query.user_id, query.limit).await?;
    Ok(Json(history))
}

// ============================================================================
// Playlists
// ============================================================================

#[derive(Debug, Deserialize)]
struct PlaylistQuery {
    user_id: Uuid
}

/// List user's playlists
async fn list_playlists(
    State(state): State<AppState>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<Json<Vec<SongPlaylist>>> {
    let playlists = state.songs.list_playlists(query.user_id).await?;
    Ok(Json(playlists))
}

/// Create a new playlist
async fn create_playlist(
    State(state): State<AppState>,
    Query(query): Query<PlaylistQuery>,
    Json(playlist): Json<CreatePlaylist>
) -> AppResult<Json<SongPlaylist>> {
    let created = state.songs.create_playlist(query.user_id, playlist).await?;
    Ok(Json(created))
}

/// Get playlist by ID
async fn get_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<Json<SongPlaylist>> {
    let playlist = state.songs.get_playlist(id, query.user_id).await?;
    Ok(Json(playlist))
}

/// Get playlist songs
async fn get_playlist_songs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<Json<Vec<PlaylistItem>>> {
    let items = state.songs.get_playlist_items(id, query.user_id).await?;
    Ok(Json(items))
}

/// Add song to playlist
async fn add_to_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(item): Json<AddToPlaylist>
) -> AppResult<()> {
    state.songs.add_to_playlist(id, item).await?;
    Ok(())
}

/// Remove song from playlist
async fn remove_from_playlist(
    State(state): State<AppState>,
    Path((playlist_id, item_id)): Path<(Uuid, Uuid)>
) -> AppResult<()> {
    state
        .songs
        .remove_from_playlist(playlist_id, item_id)
        .await?;
    Ok(())
}

/// Delete playlist
async fn delete_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<()> {
    state.songs.delete_playlist(id, query.user_id).await?;
    Ok(())
}
