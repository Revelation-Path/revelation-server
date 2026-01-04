// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! User notes on Bible verses.

use chrono::{DateTime, Utc};
use entity_derive::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User note attached to a Bible verse.
///
/// Allows users to write personal reflections and study notes.
#[derive(Debug, Clone, Serialize, Deserialize, Entity)]
#[entity(table = "bible_notes", sql = "full")]
pub struct Note {
    /// Unique note ID
    #[id]
    pub id: Uuid,

    /// User who created this note
    #[field(create, response)]
    pub user_id: Uuid,

    /// Verse this note is attached to
    #[field(create, response)]
    pub verse_id: i32,

    /// Note content (markdown supported)
    #[field(create, update, response)]
    #[validate(length(min = 1, max = 50000))]
    pub content: String,

    /// Whether note is private or can be shared
    #[field(create, update, response)]
    pub is_private: bool,

    /// When note was created
    #[field(response)]
    #[auto]
    pub created_at: DateTime<Utc>,

    /// When note was last updated
    #[field(response)]
    #[auto]
    pub updated_at: DateTime<Utc>
}
