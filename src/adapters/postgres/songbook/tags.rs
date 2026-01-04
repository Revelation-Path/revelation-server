// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use masterror::AppResult;
use revelation_songbook::{SongTag, ports::SongTags};
use sqlx::PgPool;

/// PostgreSQL implementation of SongTags
pub struct PgSongTags {
    pool: PgPool
}

impl PgSongTags {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl SongTags for PgSongTags {
    async fn list_tags(&self) -> AppResult<Vec<SongTag>> {
        let tags = sqlx::query_as!(
            SongTag,
            r#"
            SELECT id, name, name_ru, usage_count
            FROM song_tags
            ORDER BY usage_count DESC, name_ru
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }
}
