//! Revelation Server Library
//!
//! Exports adapters and services for use by other crates.

pub mod adapters;
pub mod domain;
pub mod loader;
pub mod services;

pub use domain::*;
pub use loader::{BibleLoader, LoadStats};
pub use services::{BibleService, NotificationService, SongbookService};
