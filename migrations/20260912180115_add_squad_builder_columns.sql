-- Player price, in tenths of a million (e.g. 125 = 12.5m), as synced from FPL.
ALTER TABLE players
ADD COLUMN IF NOT EXISTS now_cost INTEGER NOT NULL DEFAULT 0;

-- Starting XI vs bench, so the points engine can score only starters.
ALTER TABLE fantasy_team_players
ADD COLUMN IF NOT EXISTS is_starting BOOLEAN NOT NULL DEFAULT TRUE;

ALTER TABLE fantasy_team_players
ADD COLUMN IF NOT EXISTS bench_order SMALLINT;
