//! User reading progress tracking.

use chrono::{DateTime, Utc};
use entity_derive::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User's reading progress for a Bible book.
///
/// Tracks which chapter the user last read in each book.
#[derive(Debug, Clone, Serialize, Deserialize, Entity)]
#[entity(table = "bible_reading_progress", sql = "full")]
pub struct ReadingProgress {
    /// Unique progress record ID
    #[id]
    pub id: Uuid,

    /// User whose progress this tracks
    #[field(create, response)]
    pub user_id: Uuid,

    /// Book ID (1-66)
    #[field(create, update, response)]
    pub book_id: i16,

    /// Last read chapter
    #[field(create, update, response)]
    pub last_chapter: i16,

    /// Last read verse (optional, for precise tracking)
    #[field(create, update, response)]
    pub last_verse: Option<i16>,

    /// When this book was last read
    #[field(response)]
    #[auto]
    pub last_read_at: DateTime<Utc>,

    /// Total reading time in seconds (cumulative)
    #[field(update, response)]
    pub total_reading_time: Option<i64>
}

/// Reading streak for gamification.
///
/// Tracks consecutive days of Bible reading.
#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct ReadingStreak {
    /// User ID
    pub user_id:        Uuid,
    /// Current streak in days
    pub current_streak: i32,
    /// Longest streak ever achieved
    pub longest_streak: i32,
    /// Last reading date
    pub last_read_date: chrono::NaiveDate
}
