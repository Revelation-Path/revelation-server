use masterror::AppResult;
use revelation_songbook::{
    AddToPlaylist, CreatePlaylist, PlaylistItem, SongCategory, SongPlaylist, SongSummary,
    ports::PlaylistRepository
};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of PlaylistRepository
pub struct PgPlaylistRepository {
    pool: PgPool
}

impl PgPlaylistRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[derive(sqlx::FromRow)]
struct PlaylistItemRow {
    id:                  Uuid,
    position:            i16,
    transpose_semitones: i16,
    notes:               Option<String>,
    song_id:             Uuid,
    songbook_id:         Option<Uuid>,
    songbook_code:       Option<String>,
    number:              Option<i32>,
    title:               String,
    author_lyrics:       Option<String>,
    first_line:          String,
    original_key:        Option<String>,
    has_chords:          bool,
    views_count:         i32,
    favorites_count:     i32,
    is_favorite:         bool,
    categories:          Option<Vec<SongCategory>>
}

impl From<PlaylistItemRow> for PlaylistItem {
    fn from(row: PlaylistItemRow) -> Self {
        Self {
            id:                  row.id,
            song:                SongSummary {
                id:              row.song_id,
                songbook_id:     row.songbook_id,
                songbook_code:   row.songbook_code,
                number:          row.number,
                title:           row.title,
                author_lyrics:   row.author_lyrics,
                first_line:      row.first_line,
                original_key:    row.original_key,
                has_chords:      row.has_chords,
                categories:      row.categories.unwrap_or_default(),
                is_favorite:     row.is_favorite,
                views_count:     row.views_count,
                favorites_count: row.favorites_count
            },
            position:            row.position,
            transpose_semitones: row.transpose_semitones,
            notes:               row.notes
        }
    }
}

impl PlaylistRepository for PgPlaylistRepository {
    async fn list_playlists(&self, user_id: Uuid) -> AppResult<Vec<SongPlaylist>> {
        let playlists = sqlx::query_as!(
            SongPlaylist,
            r#"
            SELECT
                p.id, p.user_id, p.church_id, p.name, p.description, p.is_public, p.event_date,
                (SELECT COUNT(*) FROM song_playlist_items WHERE playlist_id = p.id)::int as "songs_count!",
                p.created_at, p.updated_at
            FROM song_playlists p
            WHERE p.user_id = $1
            ORDER BY p.updated_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(playlists)
    }

    async fn create_playlist(
        &self,
        user_id: Uuid,
        playlist: CreatePlaylist
    ) -> AppResult<SongPlaylist> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO song_playlists (user_id, church_id, name, description, is_public, event_date)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(playlist.church_id)
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(playlist.is_public)
        .bind(playlist.event_date)
        .fetch_one(&self.pool)
        .await?;

        self.get_playlist(id, user_id).await
    }

    async fn get_playlist(&self, id: Uuid, user_id: Uuid) -> AppResult<SongPlaylist> {
        let playlist = sqlx::query_as!(
            SongPlaylist,
            r#"
            SELECT
                p.id, p.user_id, p.church_id, p.name, p.description, p.is_public, p.event_date,
                (SELECT COUNT(*) FROM song_playlist_items WHERE playlist_id = p.id)::int as "songs_count!",
                p.created_at, p.updated_at
            FROM song_playlists p
            WHERE p.id = $1 AND (p.user_id = $2 OR p.is_public = true)
            "#,
            id,
            user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(playlist)
    }

    async fn get_playlist_items(
        &self,
        playlist_id: Uuid,
        user_id: Uuid
    ) -> AppResult<Vec<PlaylistItem>> {
        let items = sqlx::query_as::<_, PlaylistItemRow>(
            r#"
            SELECT
                pi.id, pi.position, pi.transpose_semitones, pi.notes,
                s.id as song_id, s.songbook_id, sb.code as songbook_code, s.number, s.title,
                s.author_lyrics, s.first_line, s.original_key, s.has_chords,
                s.views_count, s.favorites_count,
                CASE WHEN uf.user_id IS NOT NULL THEN true ELSE false END as is_favorite,
                ARRAY_AGG(DISTINCT sc.category) FILTER (WHERE sc.category IS NOT NULL) as categories
            FROM song_playlist_items pi
            JOIN songs s ON pi.song_id = s.id
            LEFT JOIN songbooks sb ON s.songbook_id = sb.id
            LEFT JOIN song_categories sc ON s.id = sc.song_id
            LEFT JOIN user_favorite_songs uf ON s.id = uf.song_id AND uf.user_id = $2
            WHERE pi.playlist_id = $1
            GROUP BY pi.id, s.id, sb.code, uf.user_id
            ORDER BY pi.position
            "#,
        )
        .bind(playlist_id)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(items.into_iter().map(|r| r.into()).collect())
    }

    async fn add_to_playlist(&self, playlist_id: Uuid, item: AddToPlaylist) -> AppResult<()> {
        let next_pos = sqlx::query_scalar::<_, i16>(
            "SELECT COALESCE(MAX(position), 0) + 1 FROM song_playlist_items WHERE playlist_id = $1"
        )
        .bind(playlist_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO song_playlist_items (playlist_id, song_id, position, transpose_semitones, notes)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(playlist_id)
        .bind(item.song_id)
        .bind(next_pos)
        .bind(item.transpose_semitones.unwrap_or(0))
        .bind(&item.notes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_from_playlist(&self, playlist_id: Uuid, item_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM song_playlist_items WHERE id = $1 AND playlist_id = $2")
            .bind(item_id)
            .bind(playlist_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete_playlist(&self, id: Uuid, user_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM song_playlists WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
