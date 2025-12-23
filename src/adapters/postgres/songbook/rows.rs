use revelation_shared::{SongCategory, SongSearchResult, SongSummary};
use uuid::Uuid;

/// Row type for song summary queries
#[derive(sqlx::FromRow)]
pub struct SongSummaryRow {
    pub id:              Uuid,
    pub songbook_id:     Option<Uuid>,
    pub songbook_code:   Option<String>,
    pub number:          Option<i32>,
    pub title:           String,
    pub author_lyrics:   Option<String>,
    pub first_line:      String,
    pub original_key:    Option<String>,
    pub has_chords:      bool,
    pub views_count:     i32,
    pub favorites_count: i32,
    pub is_favorite:     bool,
    pub categories:      Option<Vec<SongCategory>>
}

impl From<SongSummaryRow> for SongSummary {
    fn from(row: SongSummaryRow) -> Self {
        Self {
            id:              row.id,
            songbook_id:     row.songbook_id,
            songbook_code:   row.songbook_code,
            number:          row.number,
            title:           row.title,
            author_lyrics:   row.author_lyrics,
            first_line:      row.first_line,
            original_key:    row.original_key,
            has_chords:      row.has_chords,
            categories:      row.categories.unwrap_or_default(),
            is_favorite:     row.is_favorite,
            views_count:     row.views_count,
            favorites_count: row.favorites_count
        }
    }
}

/// Row type for search queries
#[derive(sqlx::FromRow)]
pub struct SongSearchRow {
    pub id:              Uuid,
    pub songbook_id:     Option<Uuid>,
    pub songbook_code:   Option<String>,
    pub songbook_name:   Option<String>,
    pub number:          Option<i32>,
    pub title:           String,
    pub author_lyrics:   Option<String>,
    pub first_line:      String,
    pub original_key:    Option<String>,
    pub has_chords:      bool,
    pub views_count:     i32,
    pub favorites_count: i32,
    pub is_favorite:     bool,
    pub categories:      Option<Vec<SongCategory>>,
    pub rank:            f32,
    pub highlight:       Option<String>
}

impl From<SongSearchRow> for SongSearchResult {
    fn from(row: SongSearchRow) -> Self {
        Self {
            song:          SongSummary {
                id:              row.id,
                songbook_id:     row.songbook_id,
                songbook_code:   row.songbook_code,
                number:          row.number,
                title:           row.title,
                author_lyrics:   row.author_lyrics,
                first_line:      row.first_line,
                original_key:    row.original_key,
                has_chords:      row.has_chords,
                categories:      row.categories.unwrap_or_default(),
                is_favorite:     row.is_favorite,
                views_count:     row.views_count,
                favorites_count: row.favorites_count
            },
            songbook_name: row.songbook_name,
            highlight:     row.highlight,
            rank:          row.rank
        }
    }
}
