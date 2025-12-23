use chrono::{Datelike, NaiveDate};
use masterror::AppResult;
use revelation_shared::{
    DailyReading, Verse,
    ports::{ReadingPlan, VerseResponseWithUser}
};
use sqlx::PgPool;
use uuid::Uuid;

/// PostgreSQL implementation of ReadingPlan
pub struct PgReadingPlan {
    pool: PgPool
}

impl PgReadingPlan {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl ReadingPlan for PgReadingPlan {
    async fn get_for_day(&self, day: i16) -> AppResult<Option<DailyReading>> {
        let reading = sqlx::query!(
            r#"
            SELECT id, day_of_year, date
            FROM daily_readings
            WHERE day_of_year = $1
            "#,
            day
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(reading) = reading else {
            return Ok(None);
        };

        let verses = sqlx::query_as!(
            Verse,
            r#"
            SELECT v.id, v.book_id, v.chapter, v.verse, v.text
            FROM daily_reading_verses drv
            JOIN bible_verses v ON v.id = drv.verse_id
            WHERE drv.daily_reading_id = $1
            ORDER BY drv.position
            "#,
            reading.id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(Some(DailyReading {
            id: reading.id,
            day_of_year: reading.day_of_year,
            date: reading.date,
            verses
        }))
    }

    async fn get_for_date(&self, date: NaiveDate) -> AppResult<Option<DailyReading>> {
        let day = date.ordinal() as i16;
        self.get_for_day(day).await
    }

    async fn get_today(&self) -> AppResult<Option<DailyReading>> {
        let today = chrono::Utc::now().date_naive();
        self.get_for_date(today).await
    }

    async fn get_responses(
        &self,
        daily_reading_id: Uuid
    ) -> AppResult<Vec<VerseResponseWithUser>> {
        let responses = sqlx::query_as!(
            VerseResponseWithUser,
            r#"
            SELECT
                vr.id, vr.user_id,
                u.name as user_name,
                vr.content, vr.created_at
            FROM verse_responses vr
            JOIN users u ON u.id = vr.user_id
            WHERE vr.daily_reading_id = $1
            ORDER BY vr.created_at ASC
            "#,
            daily_reading_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(responses)
    }

    async fn add_response(
        &self,
        user_id: Uuid,
        daily_reading_id: Uuid,
        content: &str
    ) -> AppResult<Uuid> {
        let id = Uuid::now_v7();

        sqlx::query!(
            r#"
            INSERT INTO verse_responses (id, user_id, daily_reading_id, content)
            VALUES ($1, $2, $3, $4)
            "#,
            id,
            user_id,
            daily_reading_id,
            content
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }
}
