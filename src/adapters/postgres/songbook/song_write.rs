// SPDX-FileCopyrightText: 2025-2026 Revelation Team
//
// SPDX-License-Identifier: MIT

use masterror::AppResult;
use revelation_songbook::{
    CreateSong, Song, UpdateSong,
    ports::{SongRead, SongWrite}
};
use sqlx::PgPool;
use uuid::Uuid;

use super::song_read::PgSongRead;

/// PostgreSQL implementation of SongWrite
pub struct PgSongWrite {
    pool: PgPool
}

impl PgSongWrite {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }
}

impl SongWrite for PgSongWrite {
    async fn create_song(&self, song: CreateSong) -> AppResult<Song> {
        let content_plain = strip_chords(&song.content);
        let first_line = extract_first_line(&song.content);

        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO songs (
                songbook_id, number, title, title_alt, author_lyrics, author_music,
                translator, year_written, copyright, original_key, tempo, time_signature,
                content, content_plain, first_line, source_url
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING id
            "#
        )
        .bind(song.songbook_id)
        .bind(song.number)
        .bind(&song.title)
        .bind(&song.title_alt)
        .bind(&song.author_lyrics)
        .bind(&song.author_music)
        .bind(&song.translator)
        .bind(song.year_written)
        .bind(&song.copyright)
        .bind(&song.original_key)
        .bind(song.tempo)
        .bind(&song.time_signature)
        .bind(&song.content)
        .bind(&content_plain)
        .bind(&first_line)
        .bind(&song.source_url)
        .fetch_one(&self.pool)
        .await?;

        for category in &song.categories {
            sqlx::query("INSERT INTO song_categories (song_id, category) VALUES ($1, $2)")
                .bind(id)
                .bind(category)
                .execute(&self.pool)
                .await?;
        }

        for tag_id in &song.tag_ids {
            sqlx::query("INSERT INTO song_tag_assignments (song_id, tag_id) VALUES ($1, $2)")
                .bind(id)
                .bind(tag_id)
                .execute(&self.pool)
                .await?;
        }

        PgSongRead::new(self.pool.clone()).get_song(id, None).await
    }

    async fn update_song(&self, id: Uuid, song: UpdateSong) -> AppResult<Song> {
        let mut query = String::from("UPDATE songs SET updated_at = NOW()");
        let mut param_count = 1;

        if song.title.is_some() {
            param_count += 1;
            query.push_str(&format!(", title = ${}", param_count));
        }

        if song.content.is_some() {
            param_count += 1;
            query.push_str(&format!(", content = ${}", param_count));
            param_count += 1;
            query.push_str(&format!(", content_plain = ${}", param_count));
            param_count += 1;
            query.push_str(&format!(", first_line = ${}", param_count));
        }

        query.push_str(" WHERE id = $1");

        sqlx::query(&query).bind(id).execute(&self.pool).await?;

        if let Some(categories) = song.categories {
            sqlx::query("DELETE FROM song_categories WHERE song_id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;

            for category in categories {
                sqlx::query("INSERT INTO song_categories (song_id, category) VALUES ($1, $2)")
                    .bind(id)
                    .bind(category)
                    .execute(&self.pool)
                    .await?;
            }
        }

        if let Some(tag_ids) = song.tag_ids {
            sqlx::query("DELETE FROM song_tag_assignments WHERE song_id = $1")
                .bind(id)
                .execute(&self.pool)
                .await?;

            for tag_id in tag_ids {
                sqlx::query("INSERT INTO song_tag_assignments (song_id, tag_id) VALUES ($1, $2)")
                    .bind(id)
                    .bind(tag_id)
                    .execute(&self.pool)
                    .await?;
            }
        }

        PgSongRead::new(self.pool.clone()).get_song(id, None).await
    }

    async fn delete_song(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM songs WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

/// Strip chords from ChordPro format
fn strip_chords(content: &str) -> String {
    let mut result = String::new();
    let mut in_chord = false;

    for c in content.chars() {
        match c {
            '[' => in_chord = true,
            ']' => in_chord = false,
            _ if !in_chord => result.push(c),
            _ => {}
        }
    }

    result
}

/// Extract first line from ChordPro content
fn extract_first_line(content: &str) -> String {
    let plain = strip_chords(content);
    plain
        .lines()
        .find(|line| !line.trim().is_empty() && !line.starts_with('{'))
        .unwrap_or("")
        .trim()
        .to_string()
}
