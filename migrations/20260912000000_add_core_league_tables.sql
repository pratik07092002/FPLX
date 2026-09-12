-- Backfills tables that existed on the live database (applied once by hand
-- from migrations/schema.sql) but were never actually captured as a
-- migration — so a fresh database (a new environment, Docker, CI) never got
-- them. Everything here is the pre-session baseline shape; the later
-- squad-builder/transfers/contest-type migrations layer their columns on
-- top of it exactly as they already do against the existing database.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    wallet_address TEXT UNIQUE NOT NULL,

    nonce TEXT UNIQUE,

    last_login TIMESTAMP,
    is_team_created BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS fantasy_teams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    captain_id INTEGER NOT NULL
        REFERENCES players(id),

    vice_captain_id INTEGER NOT NULL
        REFERENCES players(id),

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE(user_id),

    CHECK (captain_id <> vice_captain_id)
);

CREATE TABLE IF NOT EXISTS fantasy_team_players (
    id BIGSERIAL PRIMARY KEY,

    fantasy_team_id UUID NOT NULL
        REFERENCES fantasy_teams(id)
        ON DELETE CASCADE,

    player_id INTEGER NOT NULL
        REFERENCES players(id),

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE (fantasy_team_id, player_id)
);

CREATE TABLE IF NOT EXISTS fantasy_leagues (
    id SERIAL PRIMARY KEY,

    league_name TEXT NOT NULL,

    created_by_user_id UUID NOT NULL REFERENCES users(id),

    created_by_user_name TEXT NOT NULL,

    league_type TEXT NOT NULL
        CHECK (league_type IN ('public','private')),

    mode TEXT DEFAULT 'free'
        CHECK (mode IN ('free','paid')),

    entry_fee NUMERIC(18,8) DEFAULT 0,

    join_code TEXT UNIQUE,

    status TEXT DEFAULT 'open'
        CHECK (
            status IN (
                'open',
                'locked',
                'pending',
                'settling',
                'paid',
                'refunded'
            )
        ),

    created_at TIMESTAMP DEFAULT NOW(),

    CONSTRAINT unique_league_name UNIQUE (league_name)
);

CREATE TABLE IF NOT EXISTS fantasy_league_participants (
    id SERIAL PRIMARY KEY,

    league_id INTEGER NOT NULL
        REFERENCES fantasy_leagues(id)
        ON DELETE CASCADE,

    user_id UUID NOT NULL
        REFERENCES users(id),

    user_name TEXT NOT NULL,

    team_data JSONB DEFAULT '{}'::jsonb,

    joined_at TIMESTAMP DEFAULT NOW(),

    CONSTRAINT unique_user_league UNIQUE (league_id, user_id)
);

CREATE TABLE IF NOT EXISTS league_payouts (
    id SERIAL PRIMARY KEY,

    league_id INTEGER NOT NULL
        REFERENCES fantasy_leagues(id),

    user_id UUID NOT NULL
        REFERENCES users(id),

    rank INTEGER NOT NULL,

    amount NUMERIC(18,8) NOT NULL,

    tx_hash TEXT,

    status TEXT DEFAULT 'pending'
        CHECK (
            status IN ('pending','paid','failed')
        ),

    created_at TIMESTAMP DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION check_league_limit()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.league_type = 'public' THEN
        IF (
            SELECT COUNT(*)
            FROM fantasy_leagues
            WHERE created_by_user_id = NEW.created_by_user_id
            AND league_type='public'
        ) >= 3 THEN
            RAISE EXCEPTION 'Public league limit reached (3)';
        END IF;
    END IF;

    IF NEW.league_type = 'private' THEN
        IF (
            SELECT COUNT(*)
            FROM fantasy_leagues
            WHERE created_by_user_id = NEW.created_by_user_id
            AND league_type='private'
        ) >= 2 THEN
            RAISE EXCEPTION 'Private league limit reached (2)';
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_check_league_limit ON fantasy_leagues;
CREATE TRIGGER trg_check_league_limit
BEFORE INSERT ON fantasy_leagues
FOR EACH ROW
EXECUTE FUNCTION check_league_limit();

CREATE INDEX IF NOT EXISTS idx_league_creator ON fantasy_leagues(created_by_user_id);
CREATE INDEX IF NOT EXISTS idx_participant_league ON fantasy_league_participants(league_id);
