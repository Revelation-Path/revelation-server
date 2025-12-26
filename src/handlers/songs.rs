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
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(paths(
    list_songbooks,
    get_songbook,
    get_songbook_editions,
    list_songbook_songs,
    list_songs,
    search_songs,
    list_categories,
    list_by_category,
    list_tags,
    get_song,
    get_song_transposed,
    create_song,
    update_song,
    delete_song
))]
pub struct SongsApiDoc;

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

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/songbooks",
    responses(
        (status = 200, description = "List of songbooks", body = Vec<Songbook>)
    )
)]
async fn list_songbooks(State(state): State<AppState>) -> AppResult<Json<Vec<Songbook>>> {
    let songbooks = state.songs.list_songbooks().await?;
    Ok(Json(songbooks))
}

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/songbooks/{id}",
    params(
        ("id" = Uuid, Path, description = "Songbook ID")
    ),
    responses(
        (status = 200, description = "Songbook details", body = Songbook),
        (status = 404, description = "Songbook not found")
    )
)]
async fn get_songbook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
) -> AppResult<Json<Songbook>> {
    let songbook = state.songs.get_songbook(id).await?;
    Ok(Json(songbook))
}

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/songbooks/{id}/editions",
    params(
        ("id" = Uuid, Path, description = "Songbook ID")
    ),
    responses(
        (status = 200, description = "Songbook editions", body = Vec<SongbookEdition>)
    )
)]
async fn get_songbook_editions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>
) -> AppResult<Json<Vec<SongbookEdition>>> {
    let editions = state.songs.get_songbook_editions(id).await?;
    Ok(Json(editions))
}

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

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/songbooks/{id}/songs",
    params(
        ("id" = Uuid, Path, description = "Songbook ID"),
        ("limit" = Option<i64>, Query, description = "Limit results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination"),
        ("user_id" = Option<Uuid>, Query, description = "User ID for favorites")
    ),
    responses(
        (status = 200, description = "Songs in songbook", body = Vec<SongSummary>)
    )
)]
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

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs",
    params(
        ("songbook_id" = Option<Uuid>, Query, description = "Filter by songbook"),
        ("category" = Option<String>, Query, description = "Filter by category"),
        ("tag_id" = Option<Uuid>, Query, description = "Filter by tag"),
        ("key" = Option<String>, Query, description = "Filter by key"),
        ("search" = Option<String>, Query, description = "Search text"),
        ("limit" = Option<i64>, Query, description = "Limit results"),
        ("offset" = Option<i64>, Query, description = "Offset"),
        ("sort_by" = Option<String>, Query, description = "Sort by: title, number, created_at"),
        ("user_id" = Option<Uuid>, Query, description = "User ID for favorites")
    ),
    responses(
        (status = 200, description = "List of songs", body = Vec<SongSummary>)
    )
)]
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

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/search",
    params(
        ("q" = String, Query, description = "Search query"),
        ("limit" = Option<i64>, Query, description = "Max results (default 50)"),
        ("user_id" = Option<Uuid>, Query, description = "User ID for favorites")
    ),
    responses(
        (status = 200, description = "Search results", body = Vec<SongSearchResult>)
    )
)]
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

#[derive(Debug, Deserialize)]
struct UserQuery {
    user_id: Option<Uuid>
}

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/{id}",
    params(
        ("id" = Uuid, Path, description = "Song ID"),
        ("user_id" = Option<Uuid>, Query, description = "User ID for favorites")
    ),
    responses(
        (status = 200, description = "Song details", body = Song),
        (status = 404, description = "Song not found")
    )
)]
async fn get_song(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<UserQuery>
) -> AppResult<Json<Song>> {
    let song = state.songs.get_song(id, query.user_id).await?;
    Ok(Json(song))
}

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/{id}/transpose/{semitones}",
    params(
        ("id" = Uuid, Path, description = "Song ID"),
        ("semitones" = i32, Path, description = "Semitones to transpose (-12 to 12)"),
        ("user_id" = Option<Uuid>, Query, description = "User ID")
    ),
    responses(
        (status = 200, description = "Transposed song", body = Song)
    )
)]
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

#[utoipa::path(
    post,
    tag = "Songs",
    path = "/api/songs",
    request_body = CreateSong,
    responses(
        (status = 200, description = "Created song", body = Song),
        (status = 400, description = "Validation error")
    ),
    security(("cookieAuth" = []))
)]
async fn create_song(
    State(state): State<AppState>,
    Json(song): Json<CreateSong>
) -> AppResult<Json<Song>> {
    let created = state.songs.create_song(song).await?;
    Ok(Json(created))
}

#[utoipa::path(
    put,
    tag = "Songs",
    path = "/api/songs/{id}",
    params(
        ("id" = Uuid, Path, description = "Song ID")
    ),
    request_body = UpdateSong,
    responses(
        (status = 200, description = "Updated song", body = Song),
        (status = 404, description = "Song not found")
    ),
    security(("cookieAuth" = []))
)]
async fn update_song(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(song): Json<UpdateSong>
) -> AppResult<Json<Song>> {
    let updated = state.songs.update_song(id, song).await?;
    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    tag = "Songs",
    path = "/api/songs/{id}",
    params(
        ("id" = Uuid, Path, description = "Song ID")
    ),
    responses(
        (status = 200, description = "Song deleted"),
        (status = 404, description = "Song not found")
    ),
    security(("cookieAuth" = []))
)]
async fn delete_song(State(state): State<AppState>, Path(id): Path<Uuid>) -> AppResult<()> {
    state.songs.delete_song(id).await?;
    Ok(())
}

// ============================================================================
// Categories & Tags
// ============================================================================

#[derive(serde::Serialize, ToSchema)]
struct CategoryInfo {
    category: String,
    name_ru:  String
}

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/categories",
    responses(
        (status = 200, description = "List of categories", body = Vec<CategoryInfo>)
    )
)]
async fn list_categories() -> Json<Vec<CategoryInfoFull>> {
    let categories = SongCategory::all()
        .iter()
        .map(|c| CategoryInfoFull {
            category: *c,
            name_ru:  c.name_ru().to_string()
        })
        .collect();

    Json(categories)
}

#[derive(serde::Serialize)]
struct CategoryInfoFull {
    category: SongCategory,
    name_ru:  String
}

#[derive(Debug, Deserialize)]
struct CategoryQuery {
    #[serde(default = "default_limit")]
    limit:   i64,
    user_id: Option<Uuid>
}

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/categories/{category}",
    params(
        ("category" = String, Path, description = "Category name"),
        ("limit" = Option<i64>, Query, description = "Max results"),
        ("user_id" = Option<Uuid>, Query, description = "User ID")
    ),
    responses(
        (status = 200, description = "Songs in category", body = Vec<SongSummary>)
    )
)]
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

#[utoipa::path(
    get,
    tag = "Songs",
    path = "/api/songs/tags",
    responses(
        (status = 200, description = "List of tags", body = Vec<SongTag>)
    )
)]
async fn list_tags(State(state): State<AppState>) -> AppResult<Json<Vec<SongTag>>> {
    let tags = state.songs.list_tags().await?;
    Ok(Json(tags))
}

// ============================================================================
// Favorites (require auth)
// ============================================================================

#[derive(Debug, Deserialize)]
struct FavoritesQuery {
    user_id: Uuid
}

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

async fn add_favorite(
    State(state): State<AppState>,
    Path(song_id): Path<Uuid>,
    Query(query): Query<AddFavoriteQuery>
) -> AppResult<()> {
    state.songs.add_favorite(query.user_id, song_id).await?;
    Ok(())
}

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

async fn list_playlists(
    State(state): State<AppState>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<Json<Vec<SongPlaylist>>> {
    let playlists = state.songs.list_playlists(query.user_id).await?;
    Ok(Json(playlists))
}

async fn create_playlist(
    State(state): State<AppState>,
    Query(query): Query<PlaylistQuery>,
    Json(playlist): Json<CreatePlaylist>
) -> AppResult<Json<SongPlaylist>> {
    let created = state.songs.create_playlist(query.user_id, playlist).await?;
    Ok(Json(created))
}

async fn get_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<Json<SongPlaylist>> {
    let playlist = state.songs.get_playlist(id, query.user_id).await?;
    Ok(Json(playlist))
}

async fn get_playlist_songs(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<Json<Vec<PlaylistItem>>> {
    let items = state.songs.get_playlist_items(id, query.user_id).await?;
    Ok(Json(items))
}

async fn add_to_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(item): Json<AddToPlaylist>
) -> AppResult<()> {
    state.songs.add_to_playlist(id, item).await?;
    Ok(())
}

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

async fn delete_playlist(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PlaylistQuery>
) -> AppResult<()> {
    state.songs.delete_playlist(id, query.user_id).await?;
    Ok(())
}
