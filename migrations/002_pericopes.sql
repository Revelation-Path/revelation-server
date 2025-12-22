-- Bible pericopes (section headings)
-- Each pericope marks a thematic section starting at a specific verse

CREATE TABLE bible_pericopes (
    id SERIAL PRIMARY KEY,
    book_id SMALLINT NOT NULL REFERENCES bible_books(id),
    chapter SMALLINT NOT NULL,
    verse SMALLINT NOT NULL DEFAULT 1,
    heading TEXT NOT NULL,
    UNIQUE(book_id, chapter, verse, heading)
);

CREATE INDEX idx_pericopes_book_chapter ON bible_pericopes(book_id, chapter);

-- Chapter verse counts (for displaying "X verses")
CREATE TABLE bible_chapter_info (
    book_id SMALLINT NOT NULL REFERENCES bible_books(id),
    chapter SMALLINT NOT NULL,
    verse_count SMALLINT NOT NULL,
    PRIMARY KEY (book_id, chapter)
);

-- Genesis chapter info
INSERT INTO bible_chapter_info (book_id, chapter, verse_count) VALUES
    (1, 1, 31), (1, 2, 25), (1, 3, 24), (1, 4, 26), (1, 5, 32),
    (1, 6, 22), (1, 7, 24), (1, 8, 22), (1, 9, 29), (1, 10, 32),
    (1, 11, 32), (1, 12, 20), (1, 13, 18), (1, 14, 24), (1, 15, 21),
    (1, 16, 16), (1, 17, 27), (1, 18, 33), (1, 19, 38), (1, 20, 18),
    (1, 21, 34), (1, 22, 24), (1, 23, 20), (1, 24, 67), (1, 25, 34),
    (1, 26, 35), (1, 27, 46), (1, 28, 22), (1, 29, 35), (1, 30, 43),
    (1, 31, 55), (1, 32, 32), (1, 33, 20), (1, 34, 31), (1, 35, 29),
    (1, 36, 43), (1, 37, 36), (1, 38, 30), (1, 39, 23), (1, 40, 23),
    (1, 41, 57), (1, 42, 38), (1, 43, 34), (1, 44, 34), (1, 45, 28),
    (1, 46, 34), (1, 47, 31), (1, 48, 22), (1, 49, 33), (1, 50, 26);

-- Genesis pericopes (sample data)
INSERT INTO bible_pericopes (book_id, chapter, verse, heading) VALUES
    -- Genesis
    (1, 1, 1, 'Сотворение мира'),
    (1, 2, 1, 'Седьмой день'),
    (1, 2, 4, 'Эдемский сад'),
    (1, 3, 1, 'Грехопадение человека'),
    (1, 3, 14, 'Божье возмездие'),
    (1, 3, 22, 'Изгнание из Эдема'),
    (1, 4, 1, 'Каин и Авель'),
    (1, 4, 17, 'Потомки Каина'),
    (1, 4, 25, 'Сиф и Енос'),
    (1, 5, 1, 'Родословие Адама'),
    (1, 6, 1, 'Развращенность человечества'),
    (1, 6, 9, 'Ной'),
    (1, 7, 1, 'Потоп'),
    (1, 8, 1, 'Окончание потопа'),
    (1, 8, 20, 'Завет Бога с Ноем'),
    (1, 9, 1, 'Благословение Ноя'),
    (1, 9, 18, 'Ной и его сыновья'),
    (1, 10, 1, 'Родословие сыновей Ноя'),
    (1, 11, 1, 'Вавилонская башня'),
    (1, 11, 10, 'Родословие Сима'),
    (1, 11, 27, 'Родословие Фарры'),
    (1, 12, 1, 'Призвание Аврама'),
    (1, 12, 10, 'Аврам в Египте'),
    (1, 13, 1, 'Разделение Аврама и Лота'),
    (1, 14, 1, 'Аврам спасает Лота'),
    (1, 14, 17, 'Мелхиседек благословляет Аврама'),
    (1, 15, 1, 'Завет Бога с Аврамом'),
    (1, 16, 1, 'Агарь и Измаил'),
    (1, 17, 1, 'Завет обрезания'),
    (1, 18, 1, 'Три гостя Авраама'),
    (1, 18, 16, 'Авраам ходатайствует за Содом'),
    (1, 19, 1, 'Гибель Содома и Гоморры'),
    (1, 19, 30, 'Лот и его дочери'),
    (1, 20, 1, 'Авраам и Авимелех'),
    (1, 21, 1, 'Рождение Исаака'),
    (1, 21, 8, 'Изгнание Агари и Измаила'),
    (1, 21, 22, 'Союз с Авимелехом'),
    (1, 22, 1, 'Жертвоприношение Исаака'),
    (1, 22, 20, 'Потомки Нахора'),
    (1, 23, 1, 'Смерть и погребение Сарры'),
    (1, 24, 1, 'Исаак и Ревекка'),
    (1, 25, 1, 'Потомки Авраама от Хеттуры'),
    (1, 25, 7, 'Смерть Авраама'),
    (1, 25, 12, 'Потомки Измаила'),
    (1, 25, 19, 'Исав и Иаков'),
    (1, 25, 29, 'Исав продает первородство'),
    (1, 26, 1, 'Исаак и Авимелех'),
    (1, 26, 34, 'Жены Исава'),
    (1, 27, 1, 'Иаков получает благословение'),
    (1, 27, 41, 'Бегство Иакова'),
    (1, 28, 10, 'Сон Иакова в Вефиле'),
    (1, 29, 1, 'Иаков у Лавана'),
    (1, 29, 31, 'Дети Иакова'),
    (1, 30, 25, 'Богатство Иакова'),
    (1, 31, 1, 'Бегство Иакова от Лавана'),
    (1, 31, 22, 'Лаван преследует Иакова'),
    (1, 31, 43, 'Договор Иакова с Лаваном'),
    (1, 32, 1, 'Иаков готовится к встрече с Исавом'),
    (1, 32, 22, 'Борьба Иакова с Богом'),
    (1, 33, 1, 'Встреча Иакова с Исавом'),
    (1, 33, 18, 'Иаков в Сихеме'),
    (1, 34, 1, 'Дина и Сихем'),
    (1, 35, 1, 'Иаков в Вефиле'),
    (1, 35, 16, 'Смерть Рахили'),
    (1, 35, 22, 'Сыновья Иакова'),
    (1, 35, 27, 'Смерть Исаака'),
    (1, 36, 1, 'Потомки Исава'),
    (1, 37, 1, 'Сны Иосифа'),
    (1, 37, 12, 'Иосиф продан в рабство'),
    (1, 38, 1, 'Иуда и Фамарь'),
    (1, 39, 1, 'Иосиф в доме Потифара'),
    (1, 39, 7, 'Иосиф и жена Потифара'),
    (1, 39, 19, 'Иосиф в темнице'),
    (1, 40, 1, 'Иосиф толкует сны узников'),
    (1, 41, 1, 'Сны фараона'),
    (1, 41, 37, 'Возвышение Иосифа'),
    (1, 41, 53, 'Начало голода'),
    (1, 42, 1, 'Братья Иосифа в Египте'),
    (1, 43, 1, 'Второе путешествие в Египет'),
    (1, 44, 1, 'Чаша в мешке Вениамина'),
    (1, 45, 1, 'Иосиф открывается братьям'),
    (1, 46, 1, 'Иаков переселяется в Египет'),
    (1, 47, 1, 'Иаков перед фараоном'),
    (1, 47, 13, 'Иосиф и голод'),
    (1, 47, 27, 'Последние годы Иакова'),
    (1, 48, 1, 'Благословение Ефрема и Манассии'),
    (1, 49, 1, 'Благословение сыновей Иакова'),
    (1, 49, 29, 'Смерть Иакова'),
    (1, 50, 1, 'Погребение Иакова'),
    (1, 50, 15, 'Последние годы Иосифа');

-- Bible footnotes (cross-references and notes)
-- Each footnote is attached to a specific verse
CREATE TABLE bible_footnotes (
    id SERIAL PRIMARY KEY,
    book_id SMALLINT NOT NULL REFERENCES bible_books(id),
    chapter SMALLINT NOT NULL,
    verse SMALLINT NOT NULL,
    marker CHAR(1) NOT NULL DEFAULT '*',
    content TEXT NOT NULL,
    UNIQUE(book_id, chapter, verse, marker)
);

CREATE INDEX idx_footnotes_book_chapter ON bible_footnotes(book_id, chapter);

-- Cross-references (parallel passages)
CREATE TABLE bible_cross_refs (
    id SERIAL PRIMARY KEY,
    from_book_id SMALLINT NOT NULL REFERENCES bible_books(id),
    from_chapter SMALLINT NOT NULL,
    from_verse SMALLINT NOT NULL,
    to_book_id SMALLINT NOT NULL REFERENCES bible_books(id),
    to_chapter SMALLINT NOT NULL,
    to_verse_start SMALLINT NOT NULL,
    to_verse_end SMALLINT
);

CREATE INDEX idx_cross_refs_from ON bible_cross_refs(from_book_id, from_chapter, from_verse);
CREATE INDEX idx_cross_refs_to ON bible_cross_refs(to_book_id, to_chapter);
