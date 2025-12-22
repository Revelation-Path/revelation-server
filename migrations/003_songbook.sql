-- Songbook: Christian hymns and worship songs with chords

-- Song categories
CREATE TYPE song_category AS ENUM (
    'praise',           -- прославление
    'worship',          -- поклонение
    'christmas',        -- рождественские
    'easter',           -- пасхальные
    'wedding',          -- свадебные
    'funeral',          -- похоронные
    'youth',            -- молодёжные
    'children',         -- детские
    'communion',        -- вечеря Господня
    'baptism',          -- крещение
    'prayer',           -- молитвенные
    'thanksgiving',     -- благодарственные
    'evangelism',       -- евангелизационные
    'repentance',       -- покаяние
    'faith',            -- вера
    'hope',             -- надежда
    'love',             -- любовь
    'second_coming',    -- второе пришествие
    'heaven',           -- небеса
    'trinity',          -- Троица
    'holy_spirit',      -- Святой Дух
    'salvation'         -- спасение
);

-- Songbooks (collections)
CREATE TABLE songbooks (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    code VARCHAR(50) NOT NULL UNIQUE,           -- 'pesn_vozrozhdeniya', 'youth_songs'
    name VARCHAR(200) NOT NULL,                 -- English name
    name_ru VARCHAR(200) NOT NULL,              -- Russian name
    description TEXT,
    cover_url VARCHAR(500),
    songs_count INTEGER NOT NULL DEFAULT 0,
    is_public BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Songs
CREATE TABLE songs (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    songbook_id UUID REFERENCES songbooks(id) ON DELETE SET NULL,
    number INTEGER,                             -- номер в сборнике (может быть NULL для песен вне сборника)

    -- Metadata
    title VARCHAR(300) NOT NULL,
    title_alt VARCHAR(300),                     -- альтернативное название
    author_lyrics VARCHAR(200),                 -- автор текста
    author_music VARCHAR(200),                  -- автор музыки
    translator VARCHAR(200),                    -- переводчик (если перевод)
    year_written SMALLINT,                      -- год написания
    copyright TEXT,                             -- информация об авторских правах

    -- Musical info
    original_key VARCHAR(10),                   -- тональность (C, Am, F#m, Bb)
    tempo INTEGER CHECK (tempo > 0 AND tempo < 300),  -- BPM
    time_signature VARCHAR(10) DEFAULT '4/4',   -- размер (4/4, 3/4, 6/8)

    -- Content (ChordPro format)
    content TEXT NOT NULL,                      -- полный текст с аккордами в формате ChordPro
    content_plain TEXT NOT NULL,                -- текст без аккордов (для поиска)

    -- Search optimization
    first_line VARCHAR(300) NOT NULL,           -- первая строка (для поиска по началу)
    content_search TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('russian', coalesce(title, '')), 'A') ||
        setweight(to_tsvector('russian', coalesce(title_alt, '')), 'A') ||
        setweight(to_tsvector('russian', coalesce(first_line, '')), 'B') ||
        setweight(to_tsvector('russian', coalesce(content_plain, '')), 'C')
    ) STORED,

    -- Source
    source_url VARCHAR(500),                    -- откуда взято

    -- Stats
    views_count INTEGER NOT NULL DEFAULT 0,
    favorites_count INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    UNIQUE NULLS NOT DISTINCT (songbook_id, number)
);

-- Indexes for songs
CREATE INDEX idx_songs_title ON songs(title);
CREATE INDEX idx_songs_title_alt ON songs(title_alt) WHERE title_alt IS NOT NULL;
CREATE INDEX idx_songs_first_line ON songs(first_line);
CREATE INDEX idx_songs_search ON songs USING GIN(content_search);
CREATE INDEX idx_songs_songbook ON songs(songbook_id) WHERE songbook_id IS NOT NULL;
CREATE INDEX idx_songs_views ON songs(views_count DESC);
CREATE INDEX idx_songs_favorites ON songs(favorites_count DESC);

-- Song categories (many-to-many)
CREATE TABLE song_categories (
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    category song_category NOT NULL,
    PRIMARY KEY (song_id, category)
);

CREATE INDEX idx_song_categories_category ON song_categories(category);

-- Song tags (flexible tagging beyond predefined categories)
CREATE TABLE song_tags (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name VARCHAR(100) NOT NULL UNIQUE,
    name_ru VARCHAR(100) NOT NULL,
    usage_count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE song_tag_assignments (
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES song_tags(id) ON DELETE CASCADE,
    PRIMARY KEY (song_id, tag_id)
);

CREATE INDEX idx_song_tag_assignments_tag ON song_tag_assignments(tag_id);

-- User favorites
CREATE TABLE user_favorite_songs (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, song_id)
);

CREATE INDEX idx_favorite_songs_user ON user_favorite_songs(user_id, created_at DESC);

-- User recent songs (history)
CREATE TABLE user_song_history (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    transpose_semitones SMALLINT NOT NULL DEFAULT 0,  -- последняя транспозиция
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_song_history_user ON user_song_history(user_id, viewed_at DESC);
CREATE INDEX idx_song_history_song ON user_song_history(song_id);

-- User playlists (setlists for worship)
CREATE TABLE song_playlists (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    church_id UUID REFERENCES churches(id) ON DELETE SET NULL,  -- может быть привязан к церкви
    name VARCHAR(200) NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT false,
    event_date DATE,                            -- дата мероприятия (если setlist)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_playlists_user ON song_playlists(user_id);
CREATE INDEX idx_playlists_church ON song_playlists(church_id) WHERE church_id IS NOT NULL;
CREATE INDEX idx_playlists_event_date ON song_playlists(event_date DESC) WHERE event_date IS NOT NULL;

-- Playlist songs with order and per-song settings
CREATE TABLE song_playlist_items (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    playlist_id UUID NOT NULL REFERENCES song_playlists(id) ON DELETE CASCADE,
    song_id UUID NOT NULL REFERENCES songs(id) ON DELETE CASCADE,
    position SMALLINT NOT NULL,
    transpose_semitones SMALLINT NOT NULL DEFAULT 0,  -- транспозиция для этого плейлиста
    notes TEXT,                                       -- заметки исполнителя
    UNIQUE(playlist_id, position)
);

CREATE INDEX idx_playlist_items_playlist ON song_playlist_items(playlist_id, position);

-- Triggers
CREATE TRIGGER update_songs_updated_at
    BEFORE UPDATE ON songs
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_songbooks_updated_at
    BEFORE UPDATE ON songbooks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_playlists_updated_at
    BEFORE UPDATE ON song_playlists
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Function to update songbook song count
CREATE OR REPLACE FUNCTION update_songbook_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.songbook_id IS NOT NULL THEN
        UPDATE songbooks SET songs_count = songs_count + 1 WHERE id = NEW.songbook_id;
    ELSIF TG_OP = 'DELETE' AND OLD.songbook_id IS NOT NULL THEN
        UPDATE songbooks SET songs_count = songs_count - 1 WHERE id = OLD.songbook_id;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.songbook_id IS DISTINCT FROM NEW.songbook_id THEN
            IF OLD.songbook_id IS NOT NULL THEN
                UPDATE songbooks SET songs_count = songs_count - 1 WHERE id = OLD.songbook_id;
            END IF;
            IF NEW.songbook_id IS NOT NULL THEN
                UPDATE songbooks SET songs_count = songs_count + 1 WHERE id = NEW.songbook_id;
            END IF;
        END IF;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_songbook_count
    AFTER INSERT OR UPDATE OR DELETE ON songs
    FOR EACH ROW
    EXECUTE FUNCTION update_songbook_count();

-- Function to update favorites count
CREATE OR REPLACE FUNCTION update_song_favorites_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE songs SET favorites_count = favorites_count + 1 WHERE id = NEW.song_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE songs SET favorites_count = favorites_count - 1 WHERE id = OLD.song_id;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_favorites_count
    AFTER INSERT OR DELETE ON user_favorite_songs
    FOR EACH ROW
    EXECUTE FUNCTION update_song_favorites_count();

-- Function to update tag usage count
CREATE OR REPLACE FUNCTION update_tag_usage_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE song_tags SET usage_count = usage_count + 1 WHERE id = NEW.tag_id;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE song_tags SET usage_count = usage_count - 1 WHERE id = OLD.tag_id;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_tag_usage
    AFTER INSERT OR DELETE ON song_tag_assignments
    FOR EACH ROW
    EXECUTE FUNCTION update_tag_usage_count();

-- Initial songbooks
INSERT INTO songbooks (id, code, name, name_ru, description) VALUES
    ('019389a0-0002-7000-8000-000000000001', 'pesn_vozrozhdeniya', 'Song of Revival', 'Песнь Возрождения',
     'Сборник духовных песен евангельских христиан. Издаётся с 1978 года. Содержит более 3300 гимнов.'),
    ('019389a0-0002-7000-8000-000000000002', 'gusli', 'Gusli', 'Гусли',
     'Сборник духовных песен, составленный И.С. Прохановым в 1902 году. Основа для "Песни Возрождения".'),
    ('019389a0-0002-7000-8000-000000000003', 'pesni_hristian', 'Songs of Christians', 'Песни Христиан',
     'Современные христианские песни прославления и поклонения.'),
    ('019389a0-0002-7000-8000-000000000004', 'hillsong', 'Hillsong', 'Hillsong',
     'Песни церкви Hillsong в русском переводе.'),
    ('019389a0-0002-7000-8000-000000000005', 'bethel', 'Bethel Music', 'Bethel Music',
     'Песни Bethel Music в русском переводе.');

-- Initial tags
INSERT INTO song_tags (name, name_ru) VALUES
    ('acoustic', 'акустика'),
    ('slow', 'медленная'),
    ('fast', 'быстрая'),
    ('classic', 'классика'),
    ('modern', 'современная'),
    ('russian_original', 'русский оригинал'),
    ('translation', 'перевод'),
    ('choir', 'хоровая'),
    ('solo', 'сольная'),
    ('instrumental', 'с аккомпанементом');
