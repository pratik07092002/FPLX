-- Three contest formats sharing the leagues/participants machinery:
--   campaign - season-long, backed by the persistent fantasy_teams squad
--   derby    - single fixture, entry squad stored on the participant row
--   round    - single gameweek, entry squad stored on the participant row
ALTER TABLE fantasy_leagues
ADD COLUMN IF NOT EXISTS contest_type TEXT NOT NULL DEFAULT 'campaign'
    CHECK (contest_type IN ('campaign', 'derby', 'round'));

ALTER TABLE fantasy_leagues
ADD COLUMN IF NOT EXISTS fixture_id INTEGER REFERENCES fixtures(id);

ALTER TABLE fantasy_leagues
ADD COLUMN IF NOT EXISTS gameweek INTEGER;

ALTER TABLE fantasy_leagues
ADD CONSTRAINT contest_scope_check CHECK (
    (contest_type = 'campaign' AND fixture_id IS NULL AND gameweek IS NULL)
    OR (contest_type = 'derby' AND fixture_id IS NOT NULL AND gameweek IS NULL)
    OR (contest_type = 'round' AND gameweek IS NOT NULL AND fixture_id IS NULL)
);
