-- Fix update_songbook_count function to use explicit schema
-- This fixes the "relation songbooks does not exist" error when search_path doesn't include public

CREATE OR REPLACE FUNCTION update_songbook_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' AND NEW.songbook_id IS NOT NULL THEN
        UPDATE public.songbooks SET songs_count = songs_count + 1 WHERE id = NEW.songbook_id;
    ELSIF TG_OP = 'DELETE' AND OLD.songbook_id IS NOT NULL THEN
        UPDATE public.songbooks SET songs_count = songs_count - 1 WHERE id = OLD.songbook_id;
    ELSIF TG_OP = 'UPDATE' THEN
        IF OLD.songbook_id IS DISTINCT FROM NEW.songbook_id THEN
            IF OLD.songbook_id IS NOT NULL THEN
                UPDATE public.songbooks SET songs_count = songs_count - 1 WHERE id = OLD.songbook_id;
            END IF;
            IF NEW.songbook_id IS NOT NULL THEN
                UPDATE public.songbooks SET songs_count = songs_count + 1 WHERE id = NEW.songbook_id;
            END IF;
        END IF;
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;
