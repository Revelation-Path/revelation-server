use masterror::AppResult;
use revelation_shared::{
    Book, ChapterInfo, DailyReading, Pericope, SearchResult, Testament, Verse
};
use sqlx::PgPool;

use crate::adapters::postgres::{PgBibleRepository, PgBibleSearch, PgReadingPlan};

/// Bible service combining all bible-related adapters
#[derive(Clone)]
pub struct BibleService {
    pool: PgPool
}

impl BibleService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }

    pub async fn get_books(&self) -> AppResult<Vec<Book>> {
        use revelation_shared::ports::BibleRepository;
        PgBibleRepository::new(self.pool.clone()).get_books().await
    }

    pub async fn get_books_by_testament(&self, testament: Testament) -> AppResult<Vec<Book>> {
        use revelation_shared::ports::BibleRepository;
        PgBibleRepository::new(self.pool.clone())
            .get_books_by_testament(testament)
            .await
    }

    pub async fn get_chapter(&self, book_id: i16, chapter: i16) -> AppResult<Vec<Verse>> {
        use revelation_shared::ports::BibleRepository;
        PgBibleRepository::new(self.pool.clone())
            .get_chapter(book_id, chapter)
            .await
    }

    pub async fn get_verse(
        &self,
        book_id: i16,
        chapter: i16,
        verse: i16
    ) -> AppResult<Option<Verse>> {
        use revelation_shared::ports::BibleRepository;
        PgBibleRepository::new(self.pool.clone())
            .get_verse(book_id, chapter, verse)
            .await
    }

    pub async fn get_pericopes(&self, book_id: i16) -> AppResult<Vec<Pericope>> {
        use revelation_shared::ports::BibleRepository;
        PgBibleRepository::new(self.pool.clone())
            .get_pericopes(book_id)
            .await
    }

    pub async fn get_chapters_info(&self, book_id: i16) -> AppResult<Vec<ChapterInfo>> {
        use revelation_shared::ports::BibleRepository;
        PgBibleRepository::new(self.pool.clone())
            .get_chapters_info(book_id)
            .await
    }

    pub async fn search(&self, query: &str, limit: i64) -> AppResult<Vec<SearchResult>> {
        use revelation_shared::ports::BibleSearch;
        PgBibleSearch::new(self.pool.clone())
            .search(query, limit)
            .await
    }

    pub async fn symphony(&self, word: &str, limit: i64) -> AppResult<Vec<SearchResult>> {
        use revelation_shared::ports::BibleSearch;
        PgBibleSearch::new(self.pool.clone())
            .symphony(word, limit)
            .await
    }

    pub async fn word_count(&self, word: &str) -> AppResult<i64> {
        use revelation_shared::ports::BibleSearch;
        PgBibleSearch::new(self.pool.clone()).word_count(word).await
    }

    pub async fn get_today(&self) -> AppResult<Option<DailyReading>> {
        use revelation_shared::ports::ReadingPlan;
        PgReadingPlan::new(self.pool.clone()).get_today().await
    }

    pub async fn get_for_day(&self, day: i16) -> AppResult<Option<DailyReading>> {
        use revelation_shared::ports::ReadingPlan;
        PgReadingPlan::new(self.pool.clone()).get_for_day(day).await
    }
}
