-- The original init migration created player_match_stats.id as a plain
-- INT PRIMARY KEY (no default), while every INSERT this codebase issues
-- omits `id` and relies on auto-increment. That only ever worked because
-- the live database's actual table (created once by hand from schema.sql)
-- already had a proper SERIAL sequence — a fresh database does not.
CREATE SEQUENCE IF NOT EXISTS player_match_stats_id_seq
    OWNED BY player_match_stats.id;

SELECT setval(
    'player_match_stats_id_seq',
    COALESCE((SELECT MAX(id) FROM player_match_stats), 1),
    (SELECT MAX(id) FROM player_match_stats) IS NOT NULL
);

ALTER TABLE player_match_stats
ALTER COLUMN id SET DEFAULT nextval('player_match_stats_id_seq');
