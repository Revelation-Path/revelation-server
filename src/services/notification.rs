use masterror::AppResult;
use revelation_shared::TelegramRecipient;
use sqlx::PgPool;

use crate::adapters::postgres::PgNotificationRepository;

/// Notification service for managing notification delivery
#[derive(Clone)]
pub struct NotificationService {
    pool: PgPool
}

impl NotificationService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }

    /// Get all users who want Telegram notifications
    pub async fn get_telegram_recipients(&self) -> AppResult<Vec<TelegramRecipient>> {
        use revelation_shared::ports::NotificationRepository;
        PgNotificationRepository::new(self.pool.clone())
            .get_telegram_recipients()
            .await
    }
}
