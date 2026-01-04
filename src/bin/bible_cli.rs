// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

//! Bible data management CLI.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use masterror::prelude::*;
use revelation_server::loader::{BibleLoader, CrossRefLoader};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "bible-cli")]
#[command(about = "Bible data management CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Subcommand)]
enum Commands {
    /// Load Bible translation from JSON file
    Load {
        /// Path to JSON file (thiagobodruk/bible format)
        #[arg(short, long)]
        file: PathBuf
    },
    /// Load cross-references from OpenBible.info TSV file
    LoadCrossRefs {
        /// Path to cross_references.txt file
        #[arg(short, long)]
        file: PathBuf
    },
    /// Show statistics about loaded data
    Stats
}

#[tokio::main]
async fn main() -> AppResult<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into())
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    match cli.command {
        Commands::Load {
            file
        } => {
            tracing::info!("Loading Bible from {:?}", file);

            let loader = BibleLoader::new(pool);
            let stats = loader.load_from_json(&file).await?;

            tracing::info!("{}", stats);
        }
        Commands::LoadCrossRefs {
            file
        } => {
            tracing::info!("Loading cross-references from {:?}", file);

            let loader = CrossRefLoader::new(pool);
            let stats = loader.load_from_file(&file).await?;

            tracing::info!("{}", stats);
        }
        Commands::Stats => {
            let books: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM bible_books")
                .fetch_one(&pool)
                .await?
                .unwrap_or(0);

            let verses: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM bible_verses")
                .fetch_one(&pool)
                .await?
                .unwrap_or(0);

            let words: i64 =
                sqlx::query_scalar!("SELECT COUNT(DISTINCT word) FROM bible_word_index")
                    .fetch_one(&pool)
                    .await?
                    .unwrap_or(0);

            let cross_refs: i64 = sqlx::query_scalar!("SELECT COUNT(*) FROM bible_cross_refs")
                .fetch_one(&pool)
                .await?
                .unwrap_or(0);

            println!("Bible Statistics:");
            println!("  Books:        {books}");
            println!("  Verses:       {verses}");
            println!("  Words:        {words}");
            println!("  Cross-refs:   {cross_refs}");
        }
    }

    Ok(())
}
