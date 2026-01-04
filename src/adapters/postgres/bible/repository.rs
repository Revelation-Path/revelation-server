// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use masterror::AppResult;
use revelation_bible::{Book, ChapterInfo, Pericope, Testament, Verse, ports::BibleRepository};
use sqlx::PgPool;

/// PostgreSQL implementation of BibleRepository
pub struct PgBibleRepository {
    pool: PgPool
}

impl PgBibleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl BibleRepository for PgBibleRepository {
    async fn get_books(&self) -> AppResult<Vec<Book>> {
        let books = sqlx::query_as!(
            Book,
            r#"
            SELECT
                id, name, name_ru, abbreviation,
                testament as "testament: Testament",
                chapters_count
            FROM bible_books
            ORDER BY id
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(books)
    }

    async fn get_books_by_testament(&self, testament: Testament) -> AppResult<Vec<Book>> {
        let books = sqlx::query_as!(
            Book,
            r#"
            SELECT
                id, name, name_ru, abbreviation,
                testament as "testament: Testament",
                chapters_count
            FROM bible_books
            WHERE testament = $1
            ORDER BY id
            "#,
            testament as Testament
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(books)
    }

    async fn get_book(&self, id: i16) -> AppResult<Option<Book>> {
        let book = sqlx::query_as!(
            Book,
            r#"
            SELECT
                id, name, name_ru, abbreviation,
                testament as "testament: Testament",
                chapters_count
            FROM bible_books
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(book)
    }

    async fn get_chapter(&self, book_id: i16, chapter: i16) -> AppResult<Vec<Verse>> {
        let verses = sqlx::query_as!(
            Verse,
            r#"
            SELECT id, book_id, chapter, verse, text
            FROM bible_verses
            WHERE book_id = $1 AND chapter = $2
            ORDER BY verse
            "#,
            book_id,
            chapter
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(verses)
    }

    async fn get_verse(&self, book_id: i16, chapter: i16, verse: i16) -> AppResult<Option<Verse>> {
        let v = sqlx::query_as!(
            Verse,
            r#"
            SELECT id, book_id, chapter, verse, text
            FROM bible_verses
            WHERE book_id = $1 AND chapter = $2 AND verse = $3
            "#,
            book_id,
            chapter,
            verse
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(v)
    }

    async fn get_verses_range(
        &self,
        book_id: i16,
        chapter: i16,
        start_verse: i16,
        end_verse: i16
    ) -> AppResult<Vec<Verse>> {
        let verses = sqlx::query_as!(
            Verse,
            r#"
            SELECT id, book_id, chapter, verse, text
            FROM bible_verses
            WHERE book_id = $1 AND chapter = $2 AND verse >= $3 AND verse <= $4
            ORDER BY verse
            "#,
            book_id,
            chapter,
            start_verse,
            end_verse
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(verses)
    }

    async fn get_pericopes(&self, book_id: i16) -> AppResult<Vec<Pericope>> {
        let pericopes = sqlx::query_as!(
            Pericope,
            r#"
            SELECT chapter, verse, heading
            FROM bible_pericopes
            WHERE book_id = $1
            ORDER BY chapter, verse
            "#,
            book_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(pericopes)
    }

    async fn get_chapters_info(&self, book_id: i16) -> AppResult<Vec<ChapterInfo>> {
        let info = sqlx::query_as!(
            ChapterInfo,
            r#"
            SELECT chapter, verse_count
            FROM bible_chapter_info
            WHERE book_id = $1
            ORDER BY chapter
            "#,
            book_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(info)
    }
}
