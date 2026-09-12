-- Per-fixture fantasy points, computed by the points engine from raw match stats.
ALTER TABLE player_match_stats
ADD COLUMN IF NOT EXISTS points INTEGER NOT NULL DEFAULT 0;
