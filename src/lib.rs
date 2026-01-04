// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! Revelation Server Library
//!
//! Exports adapters and services for use by other crates.

pub mod adapters;
pub mod domain;
pub mod loader;
pub mod services;

pub use domain::*;
pub use loader::{BibleLoader, CrossRefLoader, CrossRefStats, LoadStats};
pub use services::{BibleService, NotificationService, SongbookService};
