use masterror::AppResult;
use revelation_songbook::{
    AddToPlaylist, CreatePlaylist, CreateSong, PlaylistItem, Song, SongCategory, SongFilters,
    SongHistoryEntry, SongPlaylist, SongSearchResult, SongSummary, SongTag, Songbook,
    SongbookEdition, UpdateSong
};
use sqlx::PgPool;
use uuid::Uuid;

use crate::adapters::postgres::{
    PgPlaylistRepository, PgSongFavorites, PgSongHistory, PgSongRead, PgSongSearch, PgSongTags,
    PgSongWrite, PgSongbookRead
};

/// Songbook service combining all song-related adapters
#[derive(Clone)]
pub struct SongbookService {
    pool: PgPool
}

impl SongbookService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }

    pub async fn list_songbooks(&self) -> AppResult<Vec<Songbook>> {
        use revelation_songbook::ports::SongbookRead;
        PgSongbookRead::new(self.pool.clone())
            .list_songbooks()
            .await
    }

    pub async fn get_songbook(&self, id: Uuid) -> AppResult<Songbook> {
        use revelation_songbook::ports::SongbookRead;
        PgSongbookRead::new(self.pool.clone())
            .get_songbook(id)
            .await
    }

    pub async fn get_songbook_editions(
        &self,
        songbook_id: Uuid
    ) -> AppResult<Vec<SongbookEdition>> {
        use revelation_songbook::ports::SongbookRead;
        PgSongbookRead::new(self.pool.clone())
            .get_editions(songbook_id)
            .await
    }

    pub async fn list_songs(
        &self,
        filters: &SongFilters,
        user_id: Option<Uuid>
    ) -> AppResult<Vec<SongSummary>> {
        use revelation_songbook::ports::SongRead;
        PgSongRead::new(self.pool.clone())
            .list_songs(filters, user_id)
            .await
    }

    pub async fn get_song(&self, id: Uuid, user_id: Option<Uuid>) -> AppResult<Song> {
        use revelation_songbook::ports::SongRead;
        PgSongRead::new(self.pool.clone())
            .get_song(id, user_id)
            .await
    }

    pub async fn search_songs(
        &self,
        query: &str,
        limit: i64,
        user_id: Option<Uuid>
    ) -> AppResult<Vec<SongSearchResult>> {
        use revelation_songbook::ports::SongSearch;
        PgSongSearch::new(self.pool.clone())
            .search_songs(query, limit, user_id)
            .await
    }

    pub async fn list_by_category(
        &self,
        category: SongCategory,
        limit: i64,
        user_id: Option<Uuid>
    ) -> AppResult<Vec<SongSummary>> {
        use revelation_songbook::ports::SongSearch;
        PgSongSearch::new(self.pool.clone())
            .list_by_category(category, limit, user_id)
            .await
    }

    pub async fn create_song(&self, song: CreateSong) -> AppResult<Song> {
        use revelation_songbook::ports::SongWrite;
        PgSongWrite::new(self.pool.clone()).create_song(song).await
    }

    pub async fn update_song(&self, id: Uuid, song: UpdateSong) -> AppResult<Song> {
        use revelation_songbook::ports::SongWrite;
        PgSongWrite::new(self.pool.clone())
            .update_song(id, song)
            .await
    }

    pub async fn delete_song(&self, id: Uuid) -> AppResult<()> {
        use revelation_songbook::ports::SongWrite;
        PgSongWrite::new(self.pool.clone()).delete_song(id).await
    }

    pub async fn list_favorites(&self, user_id: Uuid) -> AppResult<Vec<SongSummary>> {
        use revelation_songbook::ports::SongFavorites;
        PgSongFavorites::new(self.pool.clone())
            .list_favorites(user_id)
            .await
    }

    pub async fn add_favorite(&self, user_id: Uuid, song_id: Uuid) -> AppResult<()> {
        use revelation_songbook::ports::SongFavorites;
        PgSongFavorites::new(self.pool.clone())
            .add_favorite(user_id, song_id)
            .await
    }

    pub async fn remove_favorite(&self, user_id: Uuid, song_id: Uuid) -> AppResult<()> {
        use revelation_songbook::ports::SongFavorites;
        PgSongFavorites::new(self.pool.clone())
            .remove_favorite(user_id, song_id)
            .await
    }

    pub async fn list_recent(
        &self,
        user_id: Uuid,
        limit: i64
    ) -> AppResult<Vec<SongHistoryEntry>> {
        use revelation_songbook::ports::SongHistory;
        PgSongHistory::new(self.pool.clone())
            .list_recent(user_id, limit)
            .await
    }

    pub async fn list_tags(&self) -> AppResult<Vec<SongTag>> {
        use revelation_songbook::ports::SongTags;
        PgSongTags::new(self.pool.clone()).list_tags().await
    }

    pub async fn list_playlists(&self, user_id: Uuid) -> AppResult<Vec<SongPlaylist>> {
        use revelation_songbook::ports::PlaylistRepository;
        PgPlaylistRepository::new(self.pool.clone())
            .list_playlists(user_id)
            .await
    }

    pub async fn create_playlist(
        &self,
        user_id: Uuid,
        playlist: CreatePlaylist
    ) -> AppResult<SongPlaylist> {
        use revelation_songbook::ports::PlaylistRepository;
        PgPlaylistRepository::new(self.pool.clone())
            .create_playlist(user_id, playlist)
            .await
    }

    pub async fn get_playlist(&self, id: Uuid, user_id: Uuid) -> AppResult<SongPlaylist> {
        use revelation_songbook::ports::PlaylistRepository;
        PgPlaylistRepository::new(self.pool.clone())
            .get_playlist(id, user_id)
            .await
    }

    pub async fn get_playlist_items(
        &self,
        playlist_id: Uuid,
        user_id: Uuid
    ) -> AppResult<Vec<PlaylistItem>> {
        use revelation_songbook::ports::PlaylistRepository;
        PgPlaylistRepository::new(self.pool.clone())
            .get_playlist_items(playlist_id, user_id)
            .await
    }

    pub async fn add_to_playlist(&self, playlist_id: Uuid, item: AddToPlaylist) -> AppResult<()> {
        use revelation_songbook::ports::PlaylistRepository;
        PgPlaylistRepository::new(self.pool.clone())
            .add_to_playlist(playlist_id, item)
            .await
    }

    pub async fn remove_from_playlist(&self, playlist_id: Uuid, item_id: Uuid) -> AppResult<()> {
        use revelation_songbook::ports::PlaylistRepository;
        PgPlaylistRepository::new(self.pool.clone())
            .remove_from_playlist(playlist_id, item_id)
            .await
    }

    pub async fn delete_playlist(&self, id: Uuid, user_id: Uuid) -> AppResult<()> {
        use revelation_songbook::ports::PlaylistRepository;
        PgPlaylistRepository::new(self.pool.clone())
            .delete_playlist(id, user_id)
            .await
    }
}
