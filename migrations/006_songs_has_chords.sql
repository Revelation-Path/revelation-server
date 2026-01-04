-- SPDX-FileCopyrightText: 2025-2026 Revelation Team
--
-- SPDX-License-Identifier: MIT

-- Add has_chords column to songs table
ALTER TABLE songs ADD COLUMN IF NOT EXISTS has_chords BOOLEAN NOT NULL DEFAULT false;
-- Update existing songs based on content containing chord markers
UPDATE songs SET has_chords = true WHERE content ~ '\[[A-G][#b]?[^/\]]*\]';
