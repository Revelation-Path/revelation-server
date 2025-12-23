use masterror::AppResult;
use revelation_songbook::{SongCategory, SongSearchResult, SongSummary, ports::SongSearch};
use sqlx::PgPool;
use uuid::Uuid;

use super::rows::{SongSearchRow, SongSummaryRow};

/// PostgreSQL implementation of SongSearch
pub struct PgSongSearch {
    pool: PgPool
}

impl PgSongSearch {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl SongSearch for PgSongSearch {
    async fn search_songs(
        &self,
        query: &str,
        limit: i64,
        user_id: Option<Uuid>
    ) -> AppResult<Vec<SongSearchResult>> {
        let limit = limit.min(100);

        let results = sqlx::query_as::<_, SongSearchRow>(
            r#"
            SELECT
                s.id, s.songbook_id, sb.code as songbook_code, sb.name_ru as songbook_name,
                s.number, s.title, s.author_lyrics, s.first_line, s.original_key, s.has_chords,
                s.views_count, s.favorites_count,
                CASE WHEN uf.user_id IS NOT NULL THEN true ELSE false END as is_favorite,
                ARRAY_AGG(DISTINCT sc.category) FILTER (WHERE sc.category IS NOT NULL) as categories,
                ts_rank(s.content_search, websearch_to_tsquery('russian', $1)) as rank,
                ts_headline('russian', s.content_plain, websearch_to_tsquery('russian', $1),
                    'StartSel=<mark>, StopSel=</mark>, MaxWords=30, MinWords=15') as highlight
            FROM songs s
            LEFT JOIN songbooks sb ON s.songbook_id = sb.id
            LEFT JOIN song_categories sc ON s.id = sc.song_id
            LEFT JOIN user_favorite_songs uf ON s.id = uf.song_id AND uf.user_id = $2
            WHERE s.content_search @@ websearch_to_tsquery('russian', $1)
               OR s.title ILIKE '%' || $1 || '%'
               OR s.first_line ILIKE '%' || $1 || '%'
            GROUP BY s.id, sb.code, sb.name_ru, uf.user_id
            ORDER BY
                CASE WHEN s.title ILIKE $1 || '%' THEN 0 ELSE 1 END,
                rank DESC, s.views_count DESC
            LIMIT $3
            "#,
        )
        .bind(query)
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    async fn list_by_category(
        &self,
        category: SongCategory,
        limit: i64,
        user_id: Option<Uuid>
    ) -> AppResult<Vec<SongSummary>> {
        let songs = sqlx::query_as::<_, SongSummaryRow>(
            r#"
            SELECT
                s.id, s.songbook_id, sb.code as songbook_code, s.number, s.title,
                s.author_lyrics, s.first_line, s.original_key, s.has_chords,
                s.views_count, s.favorites_count,
                CASE WHEN uf.user_id IS NOT NULL THEN true ELSE false END as is_favorite,
                ARRAY_AGG(DISTINCT sc2.category) FILTER (WHERE sc2.category IS NOT NULL) as categories
            FROM songs s
            JOIN song_categories sc ON s.id = sc.song_id AND sc.category = $1
            LEFT JOIN songbooks sb ON s.songbook_id = sb.id
            LEFT JOIN song_categories sc2 ON s.id = sc2.song_id
            LEFT JOIN user_favorite_songs uf ON s.id = uf.song_id AND uf.user_id = $2
            GROUP BY s.id, sb.code, uf.user_id
            ORDER BY s.views_count DESC
            LIMIT $3
            "#,
        )
        .bind(category)
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(songs.into_iter().map(|r| r.into()).collect())
    }
}
