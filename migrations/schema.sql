-- =====================================================
-- FPLX INITIAL SCHEMA
-- =====================================================
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";



-- =====================================================
-- USERS
-- (If you already have this, remove this block)
-- =====================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    wallet_address TEXT UNIQUE NOT NULL,

    nonce TEXT UNIQUE,

    last_login TIMESTAMP,
    is_team_created BOOLEAN NOT NULL DEFAULT FALSE,

    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);


-- =====================================================
-- OFFICIAL FPL TEAMS
-- =====================================================

CREATE TABLE IF NOT EXISTS teams (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    short_name VARCHAR(10) NOT NULL,
    position INTEGER NOT NULL
);



-- =====================================================
-- OFFICIAL FPL PLAYERS
-- =====================================================

CREATE TABLE IF NOT EXISTS players (

    id INTEGER PRIMARY KEY,

    first_name TEXT NOT NULL,
    second_name TEXT NOT NULL,

    photo TEXT,

    team_id INTEGER NOT NULL
        REFERENCES teams(id),

    form NUMERIC DEFAULT 0, 

    points INTEGER DEFAULT 0,
    total_points INTEGER DEFAULT 0,

    minutes_played INTEGER DEFAULT 0,

    goals_scored INTEGER DEFAULT 0,
    assists INTEGER DEFAULT 0,

    yellow_cards INTEGER DEFAULT 0,
    red_cards INTEGER DEFAULT 0,

    saves INTEGER DEFAULT 0,
    starts INTEGER DEFAULT 0,

    news TEXT,

    position INTEGER NOT NULL,

    now_cost INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMP DEFAULT NOW()
);
ALTER TABLE players
ALTER COLUMN form TYPE REAL
USING form::REAL;
-- =====================================================
-- FIXTURES
-- =====================================================

CREATE TABLE IF NOT EXISTS fixtures (
    id INTEGER PRIMARY KEY,

    gameweek INTEGER NOT NULL,

    start_time TIMESTAMP,
    end_time TIMESTAMP,

    home_team_id INTEGER REFERENCES teams(id),
    away_team_id INTEGER REFERENCES teams(id),

    started BOOLEAN DEFAULT FALSE,
    finished BOOLEAN DEFAULT FALSE,

    home_score INTEGER,
    away_score INTEGER,

    is_postponed BOOLEAN DEFAULT FALSE,
    is_void BOOLEAN DEFAULT FALSE
);



-- =====================================================
-- PLAYER MATCH STATS
-- =====================================================

CREATE TABLE IF NOT EXISTS player_match_stats (

    id SERIAL PRIMARY KEY,

    fixture_id INTEGER NOT NULL REFERENCES fixtures(id),
    player_id INTEGER NOT NULL REFERENCES players(id),

    team_side CHAR(1) CHECK (team_side IN ('h','a')),

    minutes INTEGER DEFAULT 0,
    goals INTEGER DEFAULT 0,
    assists INTEGER DEFAULT 0,

    own_goals INTEGER DEFAULT 0,

    penalties_saved INTEGER DEFAULT 0,
    penalties_missed INTEGER DEFAULT 0,

    yellow_cards INTEGER DEFAULT 0,
    red_cards INTEGER DEFAULT 0,

    saves INTEGER DEFAULT 0,

    bonus INTEGER DEFAULT 0,
    bps INTEGER DEFAULT 0,

    defensive_contribution INTEGER DEFAULT 0,

    points INTEGER NOT NULL DEFAULT 0,

    created_at TIMESTAMP DEFAULT NOW(),

    UNIQUE (fixture_id, player_id)
);



-- =====================================================
-- USER FANTASY TEAMS
-- (For user-selected 11 players)
-- =====================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS fantasy_teams (

    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    captain_id INTEGER NOT NULL
        REFERENCES players(id),

    vice_captain_id INTEGER NOT NULL
        REFERENCES players(id),

    free_transfers INTEGER NOT NULL DEFAULT 1,
    last_transfer_gameweek INTEGER,

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

    is_starting BOOLEAN NOT NULL DEFAULT TRUE,
    bench_order SMALLINT,

    created_at TIMESTAMP NOT NULL DEFAULT NOW(),

    UNIQUE (fantasy_team_id, player_id)
);

-- =====================================================
-- FANTASY TRANSFERS
-- =====================================================

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

CREATE INDEX idx_transfers_team_gw
ON fantasy_transfers(fantasy_team_id, gameweek);

-- =====================================================
-- FANTASY LEAGUES
-- =====================================================

CREATE TABLE IF NOT EXISTS fantasy_leagues (

    id SERIAL PRIMARY KEY,

    league_name TEXT NOT NULL,

    created_by_user_id UUID NOT NULL REFERENCES users(id),

    created_by_user_name TEXT NOT NULL,

    league_type TEXT NOT NULL
        CHECK (league_type IN ('public','private')),

    -- campaign = season-long (fantasy_teams), derby = single fixture,
    -- round = single gameweek (derby/round entries live on the participant row)
    contest_type TEXT NOT NULL DEFAULT 'campaign'
        CHECK (contest_type IN ('campaign','derby','round')),

    fixture_id INTEGER REFERENCES fixtures(id),
    gameweek INTEGER,

    CHECK (
        (contest_type = 'campaign' AND fixture_id IS NULL AND gameweek IS NULL)
        OR (contest_type = 'derby' AND fixture_id IS NOT NULL AND gameweek IS NULL)
        OR (contest_type = 'round' AND gameweek IS NOT NULL AND fixture_id IS NULL)
    ),

    -- free or paid league
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

    created_at TIMESTAMP DEFAULT NOW()
);



-- unique league name
ALTER TABLE fantasy_leagues
ADD CONSTRAINT unique_league_name
UNIQUE (league_name);



-- =====================================================
-- FANTASY LEAGUE PARTICIPANTS
-- =====================================================

CREATE TABLE IF NOT EXISTS fantasy_league_participants (

    id SERIAL PRIMARY KEY,

    league_id INTEGER NOT NULL
        REFERENCES fantasy_leagues(id)
        ON DELETE CASCADE,

    user_id UUID NOT NULL
        REFERENCES users(id),

    user_name TEXT NOT NULL,

    team_data JSONB DEFAULT '{}'::jsonb,

    joined_at TIMESTAMP DEFAULT NOW()
);



-- user cannot join same league twice
ALTER TABLE fantasy_league_participants
ADD CONSTRAINT unique_user_league
UNIQUE (league_id, user_id);




-- =====================================================
-- PAYOUTS (For paid leagues)
-- =====================================================

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



-- =====================================================
-- LEAGUE LIMIT TRIGGER
-- Max 3 public
-- Max 2 private
-- =====================================================

CREATE OR REPLACE FUNCTION check_league_limit()
RETURNS TRIGGER AS $$
BEGIN

    -- Public limit
    IF NEW.league_type = 'public' THEN

        IF (
            SELECT COUNT(*)
            FROM fantasy_leagues
            WHERE created_by_user_id = NEW.created_by_user_id
            AND league_type='public'
        ) >= 3 THEN

            RAISE EXCEPTION
            'Public league limit reached (3)';

        END IF;

    END IF;


    -- Private limit
    IF NEW.league_type = 'private' THEN

        IF (
            SELECT COUNT(*)
            FROM fantasy_leagues
            WHERE created_by_user_id = NEW.created_by_user_id
            AND league_type='private'
        ) >= 2 THEN

            RAISE EXCEPTION
            'Private league limit reached (2)';

        END IF;

    END IF;


    RETURN NEW;

END;
$$ LANGUAGE plpgsql;



CREATE TRIGGER trg_check_league_limit
BEFORE INSERT ON fantasy_leagues
FOR EACH ROW
EXECUTE FUNCTION check_league_limit();



-- =====================================================
-- Helpful Indexes
-- =====================================================

CREATE INDEX idx_players_team
ON players(team_id);

CREATE INDEX idx_fixture_gameweek
ON fixtures(gameweek);

CREATE INDEX idx_match_stats_player
ON player_match_stats(player_id);

CREATE INDEX idx_league_creator
ON fantasy_leagues(created_by_user_id);

CREATE INDEX idx_participant_league
ON fantasy_league_participants(league_id);