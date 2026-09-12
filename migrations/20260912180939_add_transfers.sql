ALTER TABLE fantasy_teams
ADD COLUMN IF NOT EXISTS free_transfers INTEGER NOT NULL DEFAULT 1;

ALTER TABLE fantasy_teams
ADD COLUMN IF NOT EXISTS last_transfer_gameweek INTEGER;

CREATE TABLE IF NOT EXISTS fantasy_transfers (
    id BIGSERIAL PRIMARY KEY,

    fantasy_team_id UUID NOT NULL
        REFERENCES fantasy_teams(id)
        ON DELETE CASCADE,

    gameweek INTEGER NOT NULL,

    player_out_id INTEGER NOT NULL REFERENCES players(id),
    player_in_id INTEGER NOT NULL REFERENCES players(id),

    points_cost INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transfers_team_gw
ON fantasy_transfers(fantasy_team_id, gameweek);
