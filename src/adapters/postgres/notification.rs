// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use masterror::AppResult;
use revelation_user::{TelegramRecipient, ports::NotificationRepository};
use sqlx::PgPool;

/// PostgreSQL implementation of NotificationRepository
pub struct PgNotificationRepository {
    pool: PgPool
}

impl PgNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl NotificationRepository for PgNotificationRepository {
    async fn get_telegram_recipients(&self) -> AppResult<Vec<TelegramRecipient>> {
        let ids: Vec<i64> = sqlx::query_scalar(
            "SELECT telegram_id FROM users \
             WHERE telegram_id IS NOT NULL AND notification_enabled = true"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ids
            .into_iter()
            .map(|chat_id| TelegramRecipient {
                chat_id
            })
            .collect())
    }
}
