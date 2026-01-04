-- Add weight column for cross-reference relevance scoring
ALTER TABLE bible_cross_refs ADD COLUMN IF NOT EXISTS weight INTEGER DEFAULT 0;

-- Index for sorting by weight
CREATE INDEX IF NOT EXISTS idx_cross_refs_weight ON bible_cross_refs(from_book_id, from_chapter, from_verse, weight DESC);
