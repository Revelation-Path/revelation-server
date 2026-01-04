// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use revelation_server::{BibleService, SongbookService};
use sqlx::PgPool;

/// Application state shared across all handlers
#[derive(Clone)]
pub struct AppState {
    pub pool:  PgPool,
    pub bible: BibleService,
    pub songs: SongbookService
}

impl AppState {
    pub fn new(pool: PgPool) -> Self {
        Self {
            bible: BibleService::new(pool.clone()),
            songs: SongbookService::new(pool.clone()),
            pool
        }
    }
}
