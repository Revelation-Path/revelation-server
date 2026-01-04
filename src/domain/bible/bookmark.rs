// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! User bookmark for Bible verses.

use chrono::{DateTime, Utc};
use entity_derive::Entity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Bookmark color for visual categorization
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
    sqlx::Type,
    utoipa::ToSchema,
)]
#[sqlx(type_name = "bookmark_color", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BookmarkColor {
    #[default]
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    Orange
}

/// User bookmark for a Bible verse.
///
/// Allows users to save and categorize important verses.
#[derive(Debug, Clone, Serialize, Deserialize, Entity)]
#[entity(table = "bible_bookmarks", sql = "full")]
pub struct Bookmark {
    /// Unique bookmark ID
    #[id]
    pub id: Uuid,

    /// User who created this bookmark
    #[field(create, response)]
    pub user_id: Uuid,

    /// Bookmarked verse ID
    #[field(create, response)]
    pub verse_id: i32,

    /// Optional bookmark color for categorization
    #[field(create, update, response)]
    pub color: Option<BookmarkColor>,

    /// Optional title/label for the bookmark
    #[field(create, update, response)]
    pub title: Option<String>,

    /// When bookmark was created
    #[field(response)]
    #[auto]
    pub created_at: DateTime<Utc>
}
