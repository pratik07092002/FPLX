-- Add migration script here
-- Teams
CREATE TABLE IF NOT EXISTS teams (
    id INT PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    short_name VARCHAR(10),
    position INT
);

-- Players
CREATE TABLE IF NOT EXISTS players (
    id INT PRIMARY KEY,

    first_name VARCHAR(100),
    second_name VARCHAR(100),
    photo TEXT,

    team_id INT REFERENCES teams(id),

    position INT DEFAULT 0,

    form TEXT,

    total_points INT DEFAULT 0,
    minutes_played INT DEFAULT 0,

    goals_scored INT DEFAULT 0,
    assists INT DEFAULT 0,

    yellow_cards INT DEFAULT 0,
    red_cards INT DEFAULT 0,

    saves INT DEFAULT 0,
    starts INT DEFAULT 0,

    points INT DEFAULT 0,

    news VARCHAR(255),

    created_at TIMESTAMP DEFAULT now()
);

-- Fixtures
CREATE TABLE IF NOT EXISTS fixtures (
    id INT PRIMARY KEY,

    gameweek INT NOT NULL,

    start_time TIMESTAMP,
    end_time TIMESTAMP,

    home_team_id INT REFERENCES teams(id),
    away_team_id INT REFERENCES teams(id),

    started BOOLEAN DEFAULT false,
    finished BOOLEAN DEFAULT false,

    home_score INT DEFAULT 0,
    away_score INT DEFAULT 0
);

-- Player Match Stats
CREATE TABLE IF NOT EXISTS player_match_stats (
    id INT PRIMARY KEY,

    fixture_id INT REFERENCES fixtures(id),
    player_id INT REFERENCES players(id),

    team_side CHAR(1),

    minutes INT DEFAULT 0,

    goals INT DEFAULT 0,
    assists INT DEFAULT 0,

    own_goals INT DEFAULT 0,

    penalties_saved INT DEFAULT 0,
    penalties_missed INT DEFAULT 0,

    yellow_cards INT DEFAULT 0,
    red_cards INT DEFAULT 0,

    saves INT DEFAULT 0,

    bonus INT DEFAULT 0,
    bps INT DEFAULT 0,

    defensive_contribution INT DEFAULT 0,

    created_at TIMESTAMP DEFAULT now(),

    CONSTRAINT unique_fixture_player UNIQUE (fixture_id, player_id)
);
