-- Revelation Database Schema

-- Enums
CREATE TYPE gender AS ENUM ('male', 'female');
CREATE TYPE testament AS ENUM ('old', 'new');
CREATE TYPE church_role AS ENUM ('guest', 'member', 'deacon', 'elder', 'pastor', 'admin');
CREATE TYPE post_type AS ENUM ('sermon', 'discussion', 'testimony', 'prayer', 'event');
CREATE TYPE payment_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'refunded');
CREATE TYPE payment_type AS ENUM ('donation', 'subscription', 'one_time');

-- Religions and Confessions
CREATE TABLE religions (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name VARCHAR(100) NOT NULL UNIQUE,
    name_ru VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE confessions (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    religion_id UUID NOT NULL REFERENCES religions(id),
    name VARCHAR(100) NOT NULL,
    name_ru VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(religion_id, name)
);

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY,
    name VARCHAR(100),
    gender gender,
    birth_date DATE,
    confession_id UUID REFERENCES confessions(id),
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(20) UNIQUE,
    telegram_id BIGINT UNIQUE,
    notification_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_telegram_id ON users(telegram_id) WHERE telegram_id IS NOT NULL;
CREATE INDEX idx_users_email ON users(email) WHERE email IS NOT NULL;

-- Churches
CREATE TABLE churches (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name VARCHAR(200) NOT NULL,
    city VARCHAR(100) NOT NULL,
    address VARCHAR(500),
    confession_id UUID NOT NULL REFERENCES confessions(id),
    admin_id UUID NOT NULL REFERENCES users(id),
    latitude DOUBLE PRECISION,
    longitude DOUBLE PRECISION,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_churches_city ON churches(city);
CREATE INDEX idx_churches_confession ON churches(confession_id);

-- Memberships
CREATE TABLE memberships (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    church_id UUID NOT NULL REFERENCES churches(id) ON DELETE CASCADE,
    role church_role NOT NULL DEFAULT 'guest',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, church_id)
);

CREATE INDEX idx_memberships_church ON memberships(church_id);
CREATE INDEX idx_memberships_user ON memberships(user_id);

-- Bible
CREATE TABLE bible_books (
    id SMALLINT PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    name_ru VARCHAR(50) NOT NULL,
    abbreviation VARCHAR(10) NOT NULL,
    testament testament NOT NULL,
    chapters_count SMALLINT NOT NULL
);

CREATE TABLE bible_verses (
    id SERIAL PRIMARY KEY,
    book_id SMALLINT NOT NULL REFERENCES bible_books(id),
    chapter SMALLINT NOT NULL,
    verse SMALLINT NOT NULL,
    text TEXT NOT NULL,
    text_search TSVECTOR GENERATED ALWAYS AS (to_tsvector('russian', text)) STORED,
    UNIQUE(book_id, chapter, verse)
);

CREATE INDEX idx_verses_book_chapter ON bible_verses(book_id, chapter);
CREATE INDEX idx_verses_text_search ON bible_verses USING GIN(text_search);

-- Word index for symphony (concordance)
CREATE TABLE bible_word_index (
    id SERIAL PRIMARY KEY,
    word VARCHAR(100) NOT NULL,
    verse_id INTEGER NOT NULL REFERENCES bible_verses(id) ON DELETE CASCADE,
    position SMALLINT NOT NULL
);

CREATE INDEX idx_word_index_word ON bible_word_index(word);

-- Daily readings (Bible reading plan)
CREATE TABLE daily_readings (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    day_of_year SMALLINT NOT NULL UNIQUE CHECK (day_of_year >= 1 AND day_of_year <= 366),
    date DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE daily_reading_verses (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    daily_reading_id UUID NOT NULL REFERENCES daily_readings(id) ON DELETE CASCADE,
    verse_id INTEGER NOT NULL REFERENCES bible_verses(id),
    position SMALLINT NOT NULL,
    UNIQUE(daily_reading_id, verse_id)
);

-- Verse responses (user thoughts on daily readings)
CREATE TABLE verse_responses (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    daily_reading_id UUID NOT NULL REFERENCES daily_readings(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, daily_reading_id)
);

CREATE INDEX idx_verse_responses_reading ON verse_responses(daily_reading_id);

-- Posts (feed)
CREATE TABLE posts (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    church_id UUID REFERENCES churches(id) ON DELETE CASCADE,
    post_type post_type NOT NULL,
    title VARCHAR(300) NOT NULL,
    content TEXT NOT NULL,
    media_urls TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_posts_church ON posts(church_id);
CREATE INDEX idx_posts_created ON posts(created_at DESC);

-- Post comments
CREATE TABLE post_comments (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    post_id UUID NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    author_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_comments_post ON post_comments(post_id);

-- Payments
CREATE TABLE payments (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id),
    church_id UUID REFERENCES churches(id),
    payment_type payment_type NOT NULL,
    amount BIGINT NOT NULL, -- in kopeks/cents
    currency VARCHAR(3) NOT NULL DEFAULT 'RUB',
    status payment_status NOT NULL DEFAULT 'pending',
    provider VARCHAR(50) NOT NULL,
    provider_payment_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_payments_user ON payments(user_id);
CREATE INDEX idx_payments_church ON payments(church_id) WHERE church_id IS NOT NULL;

-- Payment cards
CREATE TABLE payment_cards (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_four VARCHAR(4) NOT NULL,
    brand VARCHAR(20) NOT NULL,
    exp_month SMALLINT NOT NULL,
    exp_year SMALLINT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    provider_card_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_cards_user ON payment_cards(user_id);

-- Initial data: Religions and Confessions
INSERT INTO religions (id, name, name_ru) VALUES
    ('019389a0-0000-7000-8000-000000000001', 'christianity', 'Христианство'),
    ('019389a0-0000-7000-8000-000000000002', 'islam', 'Ислам'),
    ('019389a0-0000-7000-8000-000000000003', 'judaism', 'Иудаизм');

INSERT INTO confessions (id, religion_id, name, name_ru) VALUES
    ('019389a0-0001-7000-8000-000000000001', '019389a0-0000-7000-8000-000000000001', 'baptist', 'Баптисты'),
    ('019389a0-0001-7000-8000-000000000002', '019389a0-0000-7000-8000-000000000001', 'orthodox', 'Православные'),
    ('019389a0-0001-7000-8000-000000000003', '019389a0-0000-7000-8000-000000000001', 'catholic', 'Католики'),
    ('019389a0-0001-7000-8000-000000000004', '019389a0-0000-7000-8000-000000000001', 'pentecostal', 'Пятидесятники'),
    ('019389a0-0001-7000-8000-000000000005', '019389a0-0000-7000-8000-000000000001', 'adventist', 'Адвентисты'),
    ('019389a0-0001-7000-8000-000000000006', '019389a0-0000-7000-8000-000000000001', 'lutheran', 'Лютеране'),
    ('019389a0-0001-7000-8000-000000000007', '019389a0-0000-7000-8000-000000000001', 'reformed', 'Реформаты');

-- Bible books (Synodal translation)
INSERT INTO bible_books (id, name, name_ru, abbreviation, testament, chapters_count) VALUES
    -- Old Testament
    (1, 'Genesis', 'Бытие', 'Быт', 'old', 50),
    (2, 'Exodus', 'Исход', 'Исх', 'old', 40),
    (3, 'Leviticus', 'Левит', 'Лев', 'old', 27),
    (4, 'Numbers', 'Числа', 'Чис', 'old', 36),
    (5, 'Deuteronomy', 'Второзаконие', 'Втор', 'old', 34),
    (6, 'Joshua', 'Иисус Навин', 'Нав', 'old', 24),
    (7, 'Judges', 'Судей', 'Суд', 'old', 21),
    (8, 'Ruth', 'Руфь', 'Руф', 'old', 4),
    (9, '1 Samuel', '1 Царств', '1Цар', 'old', 31),
    (10, '2 Samuel', '2 Царств', '2Цар', 'old', 24),
    (11, '1 Kings', '3 Царств', '3Цар', 'old', 22),
    (12, '2 Kings', '4 Царств', '4Цар', 'old', 25),
    (13, '1 Chronicles', '1 Паралипоменон', '1Пар', 'old', 29),
    (14, '2 Chronicles', '2 Паралипоменон', '2Пар', 'old', 36),
    (15, 'Ezra', 'Ездра', 'Езд', 'old', 10),
    (16, 'Nehemiah', 'Неемия', 'Неем', 'old', 13),
    (17, 'Esther', 'Есфирь', 'Есф', 'old', 10),
    (18, 'Job', 'Иов', 'Иов', 'old', 42),
    (19, 'Psalms', 'Псалтирь', 'Пс', 'old', 150),
    (20, 'Proverbs', 'Притчи', 'Притч', 'old', 31),
    (21, 'Ecclesiastes', 'Екклесиаст', 'Еккл', 'old', 12),
    (22, 'Song of Solomon', 'Песнь Песней', 'Песн', 'old', 8),
    (23, 'Isaiah', 'Исаия', 'Ис', 'old', 66),
    (24, 'Jeremiah', 'Иеремия', 'Иер', 'old', 52),
    (25, 'Lamentations', 'Плач Иеремии', 'Плач', 'old', 5),
    (26, 'Ezekiel', 'Иезекииль', 'Иез', 'old', 48),
    (27, 'Daniel', 'Даниил', 'Дан', 'old', 12),
    (28, 'Hosea', 'Осия', 'Ос', 'old', 14),
    (29, 'Joel', 'Иоиль', 'Иоил', 'old', 3),
    (30, 'Amos', 'Амос', 'Ам', 'old', 9),
    (31, 'Obadiah', 'Авдий', 'Авд', 'old', 1),
    (32, 'Jonah', 'Иона', 'Ион', 'old', 4),
    (33, 'Micah', 'Михей', 'Мих', 'old', 7),
    (34, 'Nahum', 'Наум', 'Наум', 'old', 3),
    (35, 'Habakkuk', 'Аввакум', 'Авв', 'old', 3),
    (36, 'Zephaniah', 'Софония', 'Соф', 'old', 3),
    (37, 'Haggai', 'Аггей', 'Агг', 'old', 2),
    (38, 'Zechariah', 'Захария', 'Зах', 'old', 14),
    (39, 'Malachi', 'Малахия', 'Мал', 'old', 4),
    -- New Testament
    (40, 'Matthew', 'От Матфея', 'Мф', 'new', 28),
    (41, 'Mark', 'От Марка', 'Мк', 'new', 16),
    (42, 'Luke', 'От Луки', 'Лк', 'new', 24),
    (43, 'John', 'От Иоанна', 'Ин', 'new', 21),
    (44, 'Acts', 'Деяния', 'Деян', 'new', 28),
    (45, 'Romans', 'Римлянам', 'Рим', 'new', 16),
    (46, '1 Corinthians', '1 Коринфянам', '1Кор', 'new', 16),
    (47, '2 Corinthians', '2 Коринфянам', '2Кор', 'new', 13),
    (48, 'Galatians', 'Галатам', 'Гал', 'new', 6),
    (49, 'Ephesians', 'Ефесянам', 'Еф', 'new', 6),
    (50, 'Philippians', 'Филиппийцам', 'Флп', 'new', 4),
    (51, 'Colossians', 'Колоссянам', 'Кол', 'new', 4),
    (52, '1 Thessalonians', '1 Фессалоникийцам', '1Фес', 'new', 5),
    (53, '2 Thessalonians', '2 Фессалоникийцам', '2Фес', 'new', 3),
    (54, '1 Timothy', '1 Тимофею', '1Тим', 'new', 6),
    (55, '2 Timothy', '2 Тимофею', '2Тим', 'new', 4),
    (56, 'Titus', 'Титу', 'Тит', 'new', 3),
    (57, 'Philemon', 'Филимону', 'Флм', 'new', 1),
    (58, 'Hebrews', 'Евреям', 'Евр', 'new', 13),
    (59, 'James', 'Иакова', 'Иак', 'new', 5),
    (60, '1 Peter', '1 Петра', '1Пет', 'new', 5),
    (61, '2 Peter', '2 Петра', '2Пет', 'new', 3),
    (62, '1 John', '1 Иоанна', '1Ин', 'new', 5),
    (63, '2 John', '2 Иоанна', '2Ин', 'new', 1),
    (64, '3 John', '3 Иоанна', '3Ин', 'new', 1),
    (65, 'Jude', 'Иуды', 'Иуд', 'new', 1),
    (66, 'Revelation', 'Откровение', 'Откр', 'new', 22);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_posts_updated_at
    BEFORE UPDATE ON posts
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
