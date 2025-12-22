-- Extended songbook metadata

-- Add metadata columns to songbooks
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS year_first_published SMALLINT;
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS year_latest_edition SMALLINT;
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS edition_name VARCHAR(100);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS total_songs_in_print INTEGER;
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS publisher VARCHAR(200);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS editor VARCHAR(200);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS isbn VARCHAR(20);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS language VARCHAR(10) DEFAULT 'ru';
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS country VARCHAR(50);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS denomination VARCHAR(100);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS website_url VARCHAR(500);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS purchase_url VARCHAR(500);
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS history TEXT;
ALTER TABLE songbooks ADD COLUMN IF NOT EXISTS notes TEXT;

-- Songbook editions tracking (for songbooks with multiple editions)
CREATE TABLE IF NOT EXISTS songbook_editions (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    songbook_id UUID NOT NULL REFERENCES songbooks(id) ON DELETE CASCADE,
    edition_name VARCHAR(100) NOT NULL,
    year_published SMALLINT NOT NULL,
    songs_count INTEGER NOT NULL,
    publisher VARCHAR(200),
    isbn VARCHAR(20),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_songbook_editions_songbook ON songbook_editions(songbook_id, year_published DESC);

-- Populate Песнь Возрождения metadata
UPDATE songbooks SET
    year_first_published = 1978,
    year_latest_edition = 2023,
    edition_name = '5000 песен',
    total_songs_in_print = 5000,
    publisher = 'Библия для всех',
    language = 'ru',
    country = 'Россия',
    denomination = 'ЕХБ (евангельские христиане-баптисты)',
    website_url = 'https://www.bible.org.ru',
    history = 'Сборник «Песнь Возрождения» берёт начало от «Гуслей» И.С. Проханова (1902). Первое издание вышло в 1978 году. Основные издания: 2000 песен (1995), 2500 песен (2005), 2800 песен (2009, изд. Библия для всех), 3055 песен (2010), 3300 песен (2012), 5000 песен (2023). Используется в баптистских и евангельских церквях России, Украины, Беларуси и СНГ.'
WHERE code = 'pesn_vozrozhdeniya';

-- Populate Гусли metadata
UPDATE songbooks SET
    year_first_published = 1902,
    year_latest_edition = 1927,
    edition_name = '3-е издание',
    total_songs_in_print = 507,
    publisher = 'Издательство «Радуга»',
    editor = 'Иван Степанович Проханов',
    language = 'ru',
    country = 'Российская Империя',
    denomination = 'Евангельские христиане',
    history = 'Первый русскоязычный сборник евангельских гимнов, составленный И.С. Прохановым. Первое издание 1902 года содержало около 100 гимнов, третье издание 1927 года — 507 гимнов. Многие песни впоследствии вошли в «Песнь Возрождения».'
WHERE code = 'gusli';

-- Populate Hillsong metadata
UPDATE songbooks SET
    year_first_published = 1983,
    year_latest_edition = 2024,
    publisher = 'Hillsong Music Australia',
    language = 'ru',
    country = 'Австралия',
    denomination = 'Hillsong Church (пятидесятники)',
    website_url = 'https://hillsong.com/music/',
    history = 'Песни церкви Hillsong (ранее Hills Christian Life Centre) из Сиднея, Австралия. Известные песни: «Могущий Бог» (Mighty to Save), «Океаны» (Oceans), «Что за прекрасное Имя» (What a Beautiful Name). Переводы на русский выполнены различными церквями и служениями.'
WHERE code = 'hillsong';

-- Populate Bethel Music metadata
UPDATE songbooks SET
    year_first_published = 2001,
    year_latest_edition = 2024,
    publisher = 'Bethel Music',
    language = 'ru',
    country = 'США',
    denomination = 'Bethel Church (харизматы)',
    website_url = 'https://bethelmusic.com',
    history = 'Песни церкви Bethel из Реддинга, Калифорния, США. Основано в 2001 году. Известные исполнители: Brian Johnson, Jenn Johnson, Amanda Cook, Cory Asbury. Популярные песни: «Reckless Love», «Goodness of God», «Raise a Hallelujah».'
WHERE code = 'bethel';

-- Insert Песнь Возрождения editions
INSERT INTO songbook_editions (songbook_id, edition_name, year_published, songs_count, publisher, notes) VALUES
    ((SELECT id FROM songbooks WHERE code = 'pesn_vozrozhdeniya'), '2000 песен', 1995, 2000, NULL, 'Раннее издание'),
    ((SELECT id FROM songbooks WHERE code = 'pesn_vozrozhdeniya'), '2500 песен', 2005, 2500, NULL, 'Расширенное издание'),
    ((SELECT id FROM songbooks WHERE code = 'pesn_vozrozhdeniya'), '2800 песен', 2009, 2800, 'Библия для всех', 'Издательство СПб'),
    ((SELECT id FROM songbooks WHERE code = 'pesn_vozrozhdeniya'), '3055 песен', 2010, 3055, 'Библия для всех', NULL),
    ((SELECT id FROM songbooks WHERE code = 'pesn_vozrozhdeniya'), '3300 песен', 2012, 3300, 'Библия для всех', NULL),
    ((SELECT id FROM songbooks WHERE code = 'pesn_vozrozhdeniya'), '5000 песен', 2023, 5000, 'Библия для всех', 'Последнее издание')
ON CONFLICT DO NOTHING;
