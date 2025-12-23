//! Revelation Server Library
//!
//! Exports adapters and services for use by other crates.

pub mod adapters;
pub mod services;

pub use services::{BibleService, NotificationService, SongbookService};
