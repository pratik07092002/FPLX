-- Same root cause as the earlier backfill migrations: the init migration
-- created players.form as TEXT, but every query in this codebase decodes
-- it as REAL (matching what schema.sql's hand-run ALTER actually produced
-- on the live database). A fresh database crashes every request that reads
-- players.form with a wire-format decode panic. Guarded so it's a no-op on
-- a database where the column is already REAL.
DO $$
BEGIN
    IF (
        SELECT data_type FROM information_schema.columns
        WHERE table_name = 'players' AND column_name = 'form'
    ) = 'text' THEN
        ALTER TABLE players
        ALTER COLUMN form TYPE REAL
        USING NULLIF(form, '')::REAL;

        ALTER TABLE players
        ALTER COLUMN form SET DEFAULT 0;
    END IF;
END $$;
