// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use masterror::AppResult;
use revelation_bible::{SearchResult, Verse, ports::BibleSearch};
use sqlx::PgPool;

/// PostgreSQL implementation of BibleSearch
pub struct PgBibleSearch {
    pool: PgPool
}

impl PgBibleSearch {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl BibleSearch for PgBibleSearch {
    async fn search(&self, query: &str, limit: i64) -> AppResult<Vec<SearchResult>> {
        let results = sqlx::query!(
            r#"
            SELECT
                v.id, v.book_id, v.chapter, v.verse, v.text,
                b.name_ru as book_name,
                ts_headline('russian', v.text, plainto_tsquery('russian', $1)) as headline
            FROM bible_verses v
            JOIN bible_books b ON b.id = v.book_id
            WHERE v.text_search @@ plainto_tsquery('russian', $1)
            ORDER BY ts_rank(v.text_search, plainto_tsquery('russian', $1)) DESC
            LIMIT $2
            "#,
            query,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                verse:      Verse {
                    id:      r.id,
                    book_id: r.book_id,
                    chapter: r.chapter,
                    verse:   r.verse,
                    text:    r.text
                },
                book_name:  r.book_name,
                highlights: Vec::new()
            })
            .collect())
    }

    async fn symphony(&self, word: &str, limit: i64) -> AppResult<Vec<SearchResult>> {
        let results = sqlx::query!(
            r#"
            SELECT
                v.id, v.book_id, v.chapter, v.verse, v.text,
                b.name_ru as book_name
            FROM bible_word_index w
            JOIN bible_verses v ON v.id = w.verse_id
            JOIN bible_books b ON b.id = v.book_id
            WHERE w.word = lower($1)
            ORDER BY v.book_id, v.chapter, v.verse
            LIMIT $2
            "#,
            word,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| SearchResult {
                verse:      Verse {
                    id:      r.id,
                    book_id: r.book_id,
                    chapter: r.chapter,
                    verse:   r.verse,
                    text:    r.text
                },
                book_name:  r.book_name,
                highlights: Vec::new()
            })
            .collect())
    }

    async fn word_count(&self, word: &str) -> AppResult<i64> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM bible_word_index
            WHERE word = lower($1)
            "#,
            word
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}
