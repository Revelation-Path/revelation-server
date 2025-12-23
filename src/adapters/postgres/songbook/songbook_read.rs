use masterror::AppResult;
use revelation_songbook::{Songbook, SongbookEdition, ports::SongbookRead};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of SongbookRead
pub struct PgSongbookRead {
    pool: PgPool
}

impl PgSongbookRead {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl SongbookRead for PgSongbookRead {
    async fn list_songbooks(&self) -> AppResult<Vec<Songbook>> {
        let songbooks = sqlx::query_as!(
            Songbook,
            r#"
            SELECT
                sb.id, sb.code, sb.name, sb.name_ru, sb.description, sb.cover_url,
                sb.songs_count,
                COALESCE((SELECT COUNT(*) FROM songs s WHERE s.songbook_id = sb.id AND s.has_chords = true), 0)::int as "songs_with_chords_count!",
                sb.is_public,
                sb.year_first_published, sb.year_latest_edition, sb.edition_name, sb.total_songs_in_print,
                sb.publisher, sb.editor, sb.isbn, sb.language, sb.country, sb.denomination,
                sb.website_url, sb.purchase_url, sb.history, sb.notes
            FROM songbooks sb
            WHERE sb.is_public = true
            ORDER BY sb.name_ru
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(songbooks)
    }

    async fn get_songbook(&self, id: Uuid) -> AppResult<Songbook> {
        let songbook = sqlx::query_as!(
            Songbook,
            r#"
            SELECT
                sb.id, sb.code, sb.name, sb.name_ru, sb.description, sb.cover_url,
                sb.songs_count,
                COALESCE((SELECT COUNT(*) FROM songs s WHERE s.songbook_id = sb.id AND s.has_chords = true), 0)::int as "songs_with_chords_count!",
                sb.is_public,
                sb.year_first_published, sb.year_latest_edition, sb.edition_name, sb.total_songs_in_print,
                sb.publisher, sb.editor, sb.isbn, sb.language, sb.country, sb.denomination,
                sb.website_url, sb.purchase_url, sb.history, sb.notes
            FROM songbooks sb
            WHERE sb.id = $1
            "#,
            id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(songbook)
    }

    async fn get_songbook_by_code(&self, code: &str) -> AppResult<Songbook> {
        let songbook = sqlx::query_as!(
            Songbook,
            r#"
            SELECT
                sb.id, sb.code, sb.name, sb.name_ru, sb.description, sb.cover_url,
                sb.songs_count,
                COALESCE((SELECT COUNT(*) FROM songs s WHERE s.songbook_id = sb.id AND s.has_chords = true), 0)::int as "songs_with_chords_count!",
                sb.is_public,
                sb.year_first_published, sb.year_latest_edition, sb.edition_name, sb.total_songs_in_print,
                sb.publisher, sb.editor, sb.isbn, sb.language, sb.country, sb.denomination,
                sb.website_url, sb.purchase_url, sb.history, sb.notes
            FROM songbooks sb
            WHERE sb.code = $1
            "#,
            code
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(songbook)
    }

    async fn get_editions(&self, songbook_id: Uuid) -> AppResult<Vec<SongbookEdition>> {
        let editions = sqlx::query_as!(
            SongbookEdition,
            r#"
            SELECT id, songbook_id, edition_name, year_published, songs_count, publisher, isbn, notes
            FROM songbook_editions
            WHERE songbook_id = $1
            ORDER BY year_published DESC
            "#,
            songbook_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(editions)
    }
}
