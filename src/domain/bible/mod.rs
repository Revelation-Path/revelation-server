// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! User-specific Bible data: bookmarks, notes, highlights, and reading
//! progress.
//!
//! This module provides CRUD entities for user interactions with Bible content.

mod bookmark;
mod highlight;
mod note;
mod reading_progress;

pub use bookmark::*;
pub use highlight::*;
pub use note::*;
pub use reading_progress::*;
