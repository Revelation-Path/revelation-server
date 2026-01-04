//! Bible data loader from JSON files.

use std::path::Path;

use masterror::prelude::*;
use serde::Deserialize;
use sqlx::PgPool;

/// Book abbreviation mapping to database IDs (thiagobodruk/bible format)
const BOOK_MAPPING: &[(&str, i16)] = &[
    // Old Testament
    ("gn", 1),    // Genesis
    ("ex", 2),    // Exodus
    ("lv", 3),    // Leviticus
    ("nm", 4),    // Numbers
    ("dt", 5),    // Deuteronomy
    ("js", 6),    // Joshua
    ("jud", 7),   // Judges
    ("rt", 8),    // Ruth
    ("1sm", 9),   // 1 Samuel
    ("2sm", 10),  // 2 Samuel
    ("1kgs", 11), // 1 Kings
    ("2kgs", 12), // 2 Kings
    ("1ch", 13),  // 1 Chronicles
    ("2ch", 14),  // 2 Chronicles
    ("ezr", 15),  // Ezra
    ("ne", 16),   // Nehemiah
    ("et", 17),   // Esther
    ("job", 18),  // Job
    ("ps", 19),   // Psalms
    ("prv", 20),  // Proverbs
    ("ec", 21),   // Ecclesiastes
    ("so", 22),   // Song of Solomon
    ("is", 23),   // Isaiah
    ("jr", 24),   // Jeremiah
    ("lm", 25),   // Lamentations
    ("ez", 26),   // Ezekiel
    ("dn", 27),   // Daniel
    ("ho", 28),   // Hosea
    ("jl", 29),   // Joel
    ("am", 30),   // Amos
    ("ob", 31),   // Obadiah
    ("jn", 32),   // Jonah
    ("mi", 33),   // Micah
    ("na", 34),   // Nahum
    ("hk", 35),   // Habakkuk
    ("zp", 36),   // Zephaniah
    ("hg", 37),   // Haggai
    ("zc", 38),   // Zechariah
    ("ml", 39),   // Malachi
    // New Testament
    ("mt", 40),  // Matthew
    ("mk", 41),  // Mark
    ("lk", 42),  // Luke
    ("jo", 43),  // John
    ("act", 44), // Acts
    ("rm", 45),  // Romans
    ("1co", 46), // 1 Corinthians
    ("2co", 47), // 2 Corinthians
    ("gl", 48),  // Galatians
    ("eph", 49), // Ephesians
    ("ph", 50),  // Philippians
    ("cl", 51),  // Colossians
    ("1ts", 52), // 1 Thessalonians
    ("2ts", 53), // 2 Thessalonians
    ("1tm", 54), // 1 Timothy
    ("2tm", 55), // 2 Timothy
    ("tt", 56),  // Titus
    ("phm", 57), // Philemon
    ("hb", 58),  // Hebrews
    ("jm", 59),  // James
    ("1pe", 60), // 1 Peter
    ("2pe", 61), // 2 Peter
    ("1jo", 62), // 1 John
    ("2jo", 63), // 2 John
    ("3jo", 64), // 3 John
    ("jd", 65),  // Jude
    ("re", 66)   // Revelation
];

/// JSON format from thiagobodruk/bible repository
#[derive(Debug, Deserialize)]
struct BibleBook {
    abbrev:   String,
    chapters: Vec<Vec<String>>
}

/// Loads Bible data from JSON file into the database
pub struct BibleLoader {
    pool: PgPool
}

impl BibleLoader {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }

    /// Load Bible from JSON file (thiagobodruk/bible format)
    pub async fn load_from_json(&self, path: impl AsRef<Path>) -> AppResult<LoadStats> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| AppError::internal(format!("Failed to read file: {e}")))?;

        // Strip UTF-8 BOM if present
        let content = content.strip_prefix('\u{feff}').unwrap_or(&content);

        let books: Vec<BibleBook> = serde_json::from_str(content)
            .map_err(|e| AppError::internal(format!("Failed to parse JSON: {e}")))?;

        self.load_books(&books).await
    }

    async fn load_books(&self, books: &[BibleBook]) -> AppResult<LoadStats> {
        let mut stats = LoadStats::default();

        // Clear existing verses
        sqlx::query!("DELETE FROM bible_word_index")
            .execute(&self.pool)
            .await?;
        sqlx::query!("DELETE FROM bible_verses")
            .execute(&self.pool)
            .await?;

        tracing::info!("Cleared existing verses");

        for book in books {
            let book_id = match self.get_book_id(&book.abbrev) {
                Some(id) => id,
                None => {
                    tracing::warn!("Unknown book abbreviation: {}", book.abbrev);
                    stats.skipped_books += 1;
                    continue;
                }
            };

            let verses_count = self.insert_book_verses(book_id, &book.chapters).await?;
            stats.books_loaded += 1;
            stats.verses_loaded += verses_count;

            tracing::info!(
                "Loaded book {} ({} chapters, {} verses)",
                book.abbrev,
                book.chapters.len(),
                verses_count
            );
        }

        // Build word index for search
        tracing::info!("Building word index...");
        self.build_word_index().await?;
        tracing::info!("Word index built");

        // Update chapters count in bible_books
        self.update_chapters_count().await?;

        Ok(stats)
    }

    fn get_book_id(&self, abbrev: &str) -> Option<i16> {
        BOOK_MAPPING
            .iter()
            .find(|(a, _)| a.eq_ignore_ascii_case(abbrev))
            .map(|(_, id)| *id)
    }

    async fn insert_book_verses(
        &self,
        book_id: i16,
        chapters: &[Vec<String>]
    ) -> AppResult<usize> {
        let mut count = 0;

        for (chapter_idx, verses) in chapters.iter().enumerate() {
            let chapter_num = (chapter_idx + 1) as i16;

            for (verse_idx, text) in verses.iter().enumerate() {
                let verse_num = (verse_idx + 1) as i16;

                sqlx::query!(
                    r#"
                    INSERT INTO bible_verses (book_id, chapter, verse, text)
                    VALUES ($1, $2, $3, $4)
                    ON CONFLICT (book_id, chapter, verse) DO UPDATE SET text = $4
                    "#,
                    book_id,
                    chapter_num,
                    verse_num,
                    text
                )
                .execute(&self.pool)
                .await?;

                count += 1;
            }
        }

        Ok(count)
    }

    async fn build_word_index(&self) -> AppResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO bible_word_index (word, verse_id, position)
            SELECT
                LOWER(REGEXP_REPLACE(word, '[^а-яА-ЯёЁa-zA-Z0-9]', '', 'g')) as word,
                v.id as verse_id,
                ROW_NUMBER() OVER (PARTITION BY v.id ORDER BY ordinality)::smallint as position
            FROM bible_verses v,
                 LATERAL REGEXP_SPLIT_TO_TABLE(v.text, '\s+') WITH ORDINALITY AS t(word, ordinality)
            WHERE LENGTH(REGEXP_REPLACE(word, '[^а-яА-ЯёЁa-zA-Z0-9]', '', 'g')) >= 3
            ON CONFLICT DO NOTHING
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update_chapters_count(&self) -> AppResult<()> {
        sqlx::query!(
            r#"
            UPDATE bible_books b SET chapters_count = (
                SELECT MAX(chapter) FROM bible_verses WHERE book_id = b.id
            )
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// Statistics from Bible loading operation
#[derive(Debug, Default)]
pub struct LoadStats {
    pub books_loaded:  usize,
    pub verses_loaded: usize,
    pub skipped_books: usize
}

impl std::fmt::Display for LoadStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Loaded {} books, {} verses ({} skipped)",
            self.books_loaded, self.verses_loaded, self.skipped_books
        )
    }
}

/// OpenBible.info book abbreviation mapping to database IDs
const OPENBIBLE_MAPPING: &[(&str, i16)] = &[
    // Old Testament
    ("Gen", 1),
    ("Exod", 2),
    ("Lev", 3),
    ("Num", 4),
    ("Deut", 5),
    ("Josh", 6),
    ("Judg", 7),
    ("Ruth", 8),
    ("1Sam", 9),
    ("2Sam", 10),
    ("1Kgs", 11),
    ("2Kgs", 12),
    ("1Chr", 13),
    ("2Chr", 14),
    ("Ezra", 15),
    ("Neh", 16),
    ("Esth", 17),
    ("Job", 18),
    ("Ps", 19),
    ("Prov", 20),
    ("Eccl", 21),
    ("Song", 22),
    ("Isa", 23),
    ("Jer", 24),
    ("Lam", 25),
    ("Ezek", 26),
    ("Dan", 27),
    ("Hos", 28),
    ("Joel", 29),
    ("Amos", 30),
    ("Obad", 31),
    ("Jonah", 32),
    ("Mic", 33),
    ("Nah", 34),
    ("Hab", 35),
    ("Zeph", 36),
    ("Hag", 37),
    ("Zech", 38),
    ("Mal", 39),
    // New Testament
    ("Matt", 40),
    ("Mark", 41),
    ("Luke", 42),
    ("John", 43),
    ("Acts", 44),
    ("Rom", 45),
    ("1Cor", 46),
    ("2Cor", 47),
    ("Gal", 48),
    ("Eph", 49),
    ("Phil", 50),
    ("Col", 51),
    ("1Thess", 52),
    ("2Thess", 53),
    ("1Tim", 54),
    ("2Tim", 55),
    ("Titus", 56),
    ("Phlm", 57),
    ("Heb", 58),
    ("Jas", 59),
    ("1Pet", 60),
    ("2Pet", 61),
    ("1John", 62),
    ("2John", 63),
    ("3John", 64),
    ("Jude", 65),
    ("Rev", 66)
];

/// Parsed verse reference
struct VerseRef {
    book_id:     i16,
    chapter:     i16,
    verse_start: i16,
    verse_end:   Option<i16>
}

/// Loads cross-references from OpenBible.info format
pub struct CrossRefLoader {
    pool: PgPool
}

impl CrossRefLoader {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool
        }
    }

    /// Load cross-references from OpenBible.info TSV file
    pub async fn load_from_file(&self, path: impl AsRef<Path>) -> AppResult<CrossRefStats> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| AppError::internal(format!("Failed to read file: {e}")))?;

        let mut stats = CrossRefStats::default();

        // Clear existing cross-references
        sqlx::query!("DELETE FROM bible_cross_refs")
            .execute(&self.pool)
            .await?;

        for line in content.lines() {
            // Skip header and comments
            if line.starts_with("From") || line.starts_with('#') || line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 3 {
                stats.skipped += 1;
                continue;
            }

            let from_ref = match self.parse_verse_ref(parts[0]) {
                Some(r) => r,
                None => {
                    stats.skipped += 1;
                    continue;
                }
            };

            let to_ref = match self.parse_verse_ref(parts[1]) {
                Some(r) => r,
                None => {
                    stats.skipped += 1;
                    continue;
                }
            };

            let weight: i32 = parts[2].parse().unwrap_or(0);

            if let Err(_) = sqlx::query!(
                r#"
                INSERT INTO bible_cross_refs
                    (from_book_id, from_chapter, from_verse,
                     to_book_id, to_chapter, to_verse_start, to_verse_end, weight)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
                from_ref.book_id,
                from_ref.chapter,
                from_ref.verse_start,
                to_ref.book_id,
                to_ref.chapter,
                to_ref.verse_start,
                to_ref.verse_end,
                weight
            )
            .execute(&self.pool)
            .await
            {
                stats.skipped += 1;
                continue;
            }

            stats.loaded += 1;

            if stats.loaded % 10000 == 0 {
                tracing::info!("Loaded {} cross-references...", stats.loaded);
            }
        }

        Ok(stats)
    }

    /// Parse OpenBible verse reference like "Gen.1.1" or "Ps.89.11-Ps.89.12"
    fn parse_verse_ref(&self, s: &str) -> Option<VerseRef> {
        // Handle range: "Ps.89.11-Ps.89.12"
        if s.contains('-') {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() == 2 {
                let start = self.parse_single_ref(parts[0])?;
                let end = self.parse_single_ref(parts[1])?;
                return Some(VerseRef {
                    book_id:     start.book_id,
                    chapter:     start.chapter,
                    verse_start: start.verse_start,
                    verse_end:   Some(end.verse_start)
                });
            }
        }

        self.parse_single_ref(s)
    }

    /// Parse single reference like "Gen.1.1"
    fn parse_single_ref(&self, s: &str) -> Option<VerseRef> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() < 3 {
            return None;
        }

        let book_id = OPENBIBLE_MAPPING
            .iter()
            .find(|(abbr, _)| *abbr == parts[0])
            .map(|(_, id)| *id)?;

        let chapter: i16 = parts[1].parse().ok()?;
        let verse: i16 = parts[2].parse().ok()?;

        Some(VerseRef {
            book_id,
            chapter,
            verse_start: verse,
            verse_end: None
        })
    }
}

/// Statistics from cross-reference loading
#[derive(Debug, Default)]
pub struct CrossRefStats {
    pub loaded:  usize,
    pub skipped: usize
}

impl std::fmt::Display for CrossRefStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Loaded {} cross-references ({} skipped)", self.loaded, self.skipped)
    }
}
