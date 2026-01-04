use masterror::AppResult;
use revelation_bible::{
    CrossReference, CrossReferenceExpanded, VerseRef,
    ports::BibleCrossReference
};
use sqlx::PgPool;

/// PostgreSQL implementation of BibleCrossReference
pub struct PgBibleCrossReference {
    pool: PgPool
}

impl PgBibleCrossReference {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl BibleCrossReference for PgBibleCrossReference {
    async fn get_cross_references(
        &self,
        book_id: i16,
        chapter: i16,
        verse: i16,
        limit: i64
    ) -> AppResult<Vec<CrossReferenceExpanded>> {
        let results = sqlx::query!(
            r#"
            SELECT
                cr.id,
                cr.from_book_id, cr.from_chapter, cr.from_verse,
                cr.to_book_id, cr.to_chapter, cr.to_verse_start, cr.to_verse_end,
                b.name_ru as book_name,
                COALESCE(
                    string_agg(v.text, ' ' ORDER BY v.verse),
                    ''
                ) as "text!"
            FROM bible_cross_refs cr
            JOIN bible_books b ON b.id = cr.to_book_id
            LEFT JOIN bible_verses v ON
                v.book_id = cr.to_book_id
                AND v.chapter = cr.to_chapter
                AND v.verse >= cr.to_verse_start
                AND v.verse <= COALESCE(cr.to_verse_end, cr.to_verse_start)
            WHERE cr.from_book_id = $1
              AND cr.from_chapter = $2
              AND cr.from_verse = $3
            GROUP BY cr.id, b.name_ru
            ORDER BY cr.to_book_id, cr.to_chapter, cr.to_verse_start
            LIMIT $4
            "#,
            book_id,
            chapter,
            verse,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| CrossReferenceExpanded {
                reference: CrossReference {
                    id:     r.id,
                    from:   VerseRef::single(r.from_book_id, r.from_chapter, r.from_verse),
                    to:     if r.to_verse_end.is_some() {
                        VerseRef::range(
                            r.to_book_id,
                            r.to_chapter,
                            r.to_verse_start,
                            r.to_verse_end.unwrap()
                        )
                    } else {
                        VerseRef::single(r.to_book_id, r.to_chapter, r.to_verse_start)
                    },
                    weight: None
                },
                book_name: r.book_name,
                text:      r.text
            })
            .collect())
    }

    async fn get_reverse_references(
        &self,
        book_id: i16,
        chapter: i16,
        verse: i16,
        limit: i64
    ) -> AppResult<Vec<CrossReferenceExpanded>> {
        let results = sqlx::query!(
            r#"
            SELECT
                cr.id,
                cr.from_book_id, cr.from_chapter, cr.from_verse,
                cr.to_book_id, cr.to_chapter, cr.to_verse_start, cr.to_verse_end,
                b.name_ru as book_name,
                COALESCE(v.text, '') as "text!"
            FROM bible_cross_refs cr
            JOIN bible_books b ON b.id = cr.from_book_id
            LEFT JOIN bible_verses v ON
                v.book_id = cr.from_book_id
                AND v.chapter = cr.from_chapter
                AND v.verse = cr.from_verse
            WHERE cr.to_book_id = $1
              AND cr.to_chapter = $2
              AND cr.to_verse_start <= $3
              AND COALESCE(cr.to_verse_end, cr.to_verse_start) >= $3
            ORDER BY cr.from_book_id, cr.from_chapter, cr.from_verse
            LIMIT $4
            "#,
            book_id,
            chapter,
            verse,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| CrossReferenceExpanded {
                reference: CrossReference {
                    id:     r.id,
                    from:   VerseRef::single(r.from_book_id, r.from_chapter, r.from_verse),
                    to:     if r.to_verse_end.is_some() {
                        VerseRef::range(
                            r.to_book_id,
                            r.to_chapter,
                            r.to_verse_start,
                            r.to_verse_end.unwrap()
                        )
                    } else {
                        VerseRef::single(r.to_book_id, r.to_chapter, r.to_verse_start)
                    },
                    weight: None
                },
                book_name: r.book_name,
                text:      r.text
            })
            .collect())
    }

    async fn get_chapter_references(
        &self,
        book_id: i16,
        chapter: i16
    ) -> AppResult<Vec<CrossReference>> {
        let results = sqlx::query!(
            r#"
            SELECT
                id,
                from_book_id, from_chapter, from_verse,
                to_book_id, to_chapter, to_verse_start, to_verse_end
            FROM bible_cross_refs
            WHERE from_book_id = $1
              AND from_chapter = $2
            ORDER BY from_verse, to_book_id, to_chapter, to_verse_start
            "#,
            book_id,
            chapter
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(results
            .into_iter()
            .map(|r| CrossReference {
                id:     r.id,
                from:   VerseRef::single(r.from_book_id, r.from_chapter, r.from_verse),
                to:     if r.to_verse_end.is_some() {
                    VerseRef::range(
                        r.to_book_id,
                        r.to_chapter,
                        r.to_verse_start,
                        r.to_verse_end.unwrap()
                    )
                } else {
                    VerseRef::single(r.to_book_id, r.to_chapter, r.to_verse_start)
                },
                weight: None
            })
            .collect())
    }

    async fn count_references(
        &self,
        book_id: i16,
        chapter: i16,
        verse: i16
    ) -> AppResult<i64> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM bible_cross_refs
            WHERE from_book_id = $1
              AND from_chapter = $2
              AND from_verse = $3
            "#,
            book_id,
            chapter,
            verse
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }
}
