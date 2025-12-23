use masterror::AppResult;
use revelation_shared::{SongCategory, SongHistoryEntry, SongSummary, ports::SongHistory};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of SongHistory
pub struct PgSongHistory {
    pool: PgPool
}

impl PgSongHistory {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[derive(sqlx::FromRow)]
struct SongHistoryRow {
    id:                  Uuid,
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
    categories:          Option<Vec<SongCategory>>,
    transpose_semitones: i16,
    viewed_at:           chrono::DateTime<chrono::Utc>
}

impl From<SongHistoryRow> for SongHistoryEntry {
    fn from(row: SongHistoryRow) -> Self {
        Self {
            song:                SongSummary {
                id:              row.id,
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
            transpose_semitones: row.transpose_semitones,
            viewed_at:           row.viewed_at
        }
    }
}

impl SongHistory for PgSongHistory {
    async fn list_recent(&self, user_id: Uuid, limit: i64) -> AppResult<Vec<SongHistoryEntry>> {
        let entries = sqlx::query_as::<_, SongHistoryRow>(
            r#"
            SELECT DISTINCT ON (s.id)
                s.id, s.songbook_id, sb.code as songbook_code, s.number, s.title,
                s.author_lyrics, s.first_line, s.original_key, s.has_chords,
                s.views_count, s.favorites_count,
                CASE WHEN uf.user_id IS NOT NULL THEN true ELSE false END as is_favorite,
                ARRAY_AGG(sc.category) FILTER (WHERE sc.category IS NOT NULL) OVER (PARTITION BY s.id) as categories,
                uh.transpose_semitones, uh.viewed_at
            FROM user_song_history uh
            JOIN songs s ON uh.song_id = s.id
            LEFT JOIN songbooks sb ON s.songbook_id = sb.id
            LEFT JOIN song_categories sc ON s.id = sc.song_id
            LEFT JOIN user_favorite_songs uf ON s.id = uf.song_id AND uf.user_id = $1
            WHERE uh.user_id = $1
            ORDER BY s.id, uh.viewed_at DESC
            LIMIT $2
            "#,
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(entries.into_iter().map(|r| r.into()).collect())
    }
}
