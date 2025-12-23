use masterror::AppResult;
use revelation_shared::{SongSummary, ports::SongFavorites};
use sqlx::PgPool;
use uuid::Uuid;

use super::rows::SongSummaryRow;

/// PostgreSQL implementation of SongFavorites
pub struct PgSongFavorites {
    pool: PgPool
}

impl PgSongFavorites {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl SongFavorites for PgSongFavorites {
    async fn list_favorites(&self, user_id: Uuid) -> AppResult<Vec<SongSummary>> {
        let songs = sqlx::query_as::<_, SongSummaryRow>(
            r#"
            SELECT
                s.id, s.songbook_id, sb.code as songbook_code, s.number, s.title,
                s.author_lyrics, s.first_line, s.original_key, s.has_chords,
                s.views_count, s.favorites_count, true as is_favorite,
                ARRAY_AGG(DISTINCT sc.category) FILTER (WHERE sc.category IS NOT NULL) as categories
            FROM songs s
            JOIN user_favorite_songs uf ON s.id = uf.song_id
            LEFT JOIN songbooks sb ON s.songbook_id = sb.id
            LEFT JOIN song_categories sc ON s.id = sc.song_id
            WHERE uf.user_id = $1
            GROUP BY s.id, sb.code, uf.created_at
            ORDER BY uf.created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(songs.into_iter().map(|r| r.into()).collect())
    }

    async fn add_favorite(&self, user_id: Uuid, song_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO user_favorite_songs (user_id, song_id)
            VALUES ($1, $2)
            ON CONFLICT (user_id, song_id) DO NOTHING
            "#
        )
        .bind(user_id)
        .bind(song_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_favorite(&self, user_id: Uuid, song_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM user_favorite_songs WHERE user_id = $1 AND song_id = $2")
            .bind(user_id)
            .bind(song_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
