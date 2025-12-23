use masterror::AppResult;
use revelation_songbook::{
    Song, SongCategory, SongFilters, SongSortBy, SongSummary, SongTag, ports::SongRead
};
use sqlx::PgPool;
use uuid::Uuid;

use super::rows::SongSummaryRow;

/// PostgreSQL implementation of SongRead
pub struct PgSongRead {
    pool: PgPool
}

impl PgSongRead {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

#[derive(sqlx::FromRow)]
struct SongRow {
    id:              Uuid,
    songbook_id:     Option<Uuid>,
    songbook_code:   Option<String>,
    number:          Option<i32>,
    title:           String,
    title_alt:       Option<String>,
    author_lyrics:   Option<String>,
    author_music:    Option<String>,
    translator:      Option<String>,
    year_written:    Option<i16>,
    copyright:       Option<String>,
    original_key:    Option<String>,
    tempo:           Option<i32>,
    time_signature:  Option<String>,
    content:         String,
    first_line:      String,
    views_count:     i32,
    favorites_count: i32,
    is_favorite:     bool,
    user_transpose:  i16
}

impl SongRow {
    fn into_song(self, categories: Vec<SongCategory>, tags: Vec<SongTag>) -> Song {
        Song {
            id: self.id,
            songbook_id: self.songbook_id,
            songbook_code: self.songbook_code,
            number: self.number,
            title: self.title,
            title_alt: self.title_alt,
            author_lyrics: self.author_lyrics,
            author_music: self.author_music,
            translator: self.translator,
            year_written: self.year_written,
            copyright: self.copyright,
            original_key: self.original_key,
            tempo: self.tempo,
            time_signature: self.time_signature,
            content: self.content,
            first_line: self.first_line,
            categories,
            tags,
            is_favorite: self.is_favorite,
            user_transpose: self.user_transpose,
            views_count: self.views_count,
            favorites_count: self.favorites_count
        }
    }
}

impl SongRead for PgSongRead {
    async fn list_songs(
        &self,
        filters: &SongFilters,
        user_id: Option<Uuid>
    ) -> AppResult<Vec<SongSummary>> {
        let limit = filters.limit.unwrap_or(50).min(100);
        let offset = filters.offset.unwrap_or(0);

        let order_by = match filters.sort_by.unwrap_or_default() {
            SongSortBy::Title => "s.title ASC",
            SongSortBy::Number => "s.number ASC NULLS LAST, s.title ASC",
            SongSortBy::ViewsDesc => "s.views_count DESC, s.title ASC",
            SongSortBy::FavoritesDesc => "s.favorites_count DESC, s.title ASC",
            SongSortBy::RecentlyAdded => "s.created_at DESC",
            SongSortBy::HasChordsFirst => "s.has_chords DESC, s.title ASC",
            SongSortBy::NoChordsFirst => "s.has_chords ASC, s.title ASC"
        };

        let songs = sqlx::query_as::<_, SongSummaryRow>(&format!(
            r#"
            SELECT
                s.id, s.songbook_id, sb.code as songbook_code, s.number, s.title,
                s.author_lyrics, s.first_line, s.original_key, s.has_chords,
                s.views_count, s.favorites_count,
                CASE WHEN uf.user_id IS NOT NULL THEN true ELSE false END as is_favorite,
                ARRAY_AGG(DISTINCT sc.category) FILTER (WHERE sc.category IS NOT NULL) as categories
            FROM songs s
            LEFT JOIN songbooks sb ON s.songbook_id = sb.id
            LEFT JOIN song_categories sc ON s.id = sc.song_id
            LEFT JOIN user_favorite_songs uf ON s.id = uf.song_id AND uf.user_id = $1
            WHERE 1=1
                AND ($2::uuid IS NULL OR s.songbook_id = $2)
                AND ($3::song_category IS NULL OR sc.category = $3)
                AND ($4::uuid IS NULL OR EXISTS (
                    SELECT 1 FROM song_tag_assignments sta WHERE sta.song_id = s.id AND sta.tag_id = $4
                ))
                AND ($5::text IS NULL OR s.original_key = $5)
            GROUP BY s.id, sb.code, uf.user_id
            ORDER BY {}
            LIMIT $6 OFFSET $7
            "#,
            order_by
        ))
        .bind(user_id)
        .bind(filters.songbook_id)
        .bind(filters.category)
        .bind(filters.tag_id)
        .bind(&filters.key)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(songs.into_iter().map(|r| r.into()).collect())
    }

    async fn get_song(&self, id: Uuid, user_id: Option<Uuid>) -> AppResult<Song> {
        let row = sqlx::query_as::<_, SongRow>(
            r#"
            SELECT
                s.id, s.songbook_id, s.number, s.title, s.title_alt,
                s.author_lyrics, s.author_music, s.translator, s.year_written,
                s.copyright, s.original_key, s.tempo, s.time_signature,
                s.content, s.first_line, s.views_count, s.favorites_count,
                sb.code as songbook_code,
                CASE WHEN uf.user_id IS NOT NULL THEN true ELSE false END as is_favorite,
                COALESCE(uh.transpose_semitones, 0)::smallint as user_transpose
            FROM songs s
            LEFT JOIN songbooks sb ON s.songbook_id = sb.id
            LEFT JOIN user_favorite_songs uf ON s.id = uf.song_id AND uf.user_id = $2
            LEFT JOIN (
                SELECT DISTINCT ON (song_id) song_id, transpose_semitones
                FROM user_song_history WHERE user_id = $2
                ORDER BY song_id, viewed_at DESC
            ) uh ON s.id = uh.song_id
            WHERE s.id = $1
            "#
        )
        .bind(id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let categories = sqlx::query_scalar::<_, SongCategory>(
            "SELECT category FROM song_categories WHERE song_id = $1"
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;

        let tags = sqlx::query_as!(
            SongTag,
            r#"
            SELECT t.id, t.name, t.name_ru, t.usage_count
            FROM song_tags t
            JOIN song_tag_assignments sta ON t.id = sta.tag_id
            WHERE sta.song_id = $1
            "#,
            id
        )
        .fetch_all(&self.pool)
        .await?;

        sqlx::query("UPDATE songs SET views_count = views_count + 1 WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if let Some(uid) = user_id {
            sqlx::query(
                "INSERT INTO user_song_history (user_id, song_id, viewed_at) VALUES ($1, $2, NOW())",
            )
            .bind(uid)
            .bind(id)
            .execute(&self.pool)
            .await?;
        }

        Ok(row.into_song(categories, tags))
    }

    async fn get_song_by_number(
        &self,
        songbook_id: Uuid,
        number: i32,
        user_id: Option<Uuid>
    ) -> AppResult<Song> {
        let id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM songs WHERE songbook_id = $1 AND number = $2"
        )
        .bind(songbook_id)
        .bind(number)
        .fetch_one(&self.pool)
        .await?;

        self.get_song(id, user_id).await
    }
}
