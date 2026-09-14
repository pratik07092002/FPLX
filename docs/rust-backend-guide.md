# FPLX Backend — Developer Guide

A guide to the Rust API for someone picking up this codebase for the first time. Covers structure, request flow, the main subsystems, the database, and how to actually run it.

## Stack

- **actix-web 4** — HTTP framework
- **sqlx 0.7** (Postgres, compile-time checked queries) — no ORM
- **tokio** — async runtime
- **jsonwebtoken** + **ed25519-dalek** + **bs58** — Solana wallet-signature auth
- **reqwest** (rustls, not OpenSSL — deliberate, see [Docker](#docker--deployment)) — calls the official FPL API
- **sqlx::migrate!** — embedded, auto-run migrations (no manual step needed at deploy time)

## Project layout

```
src/
  main.rs                  entry point: DB connect, migrate, CORS, routes, background sync loop
  routes/
    sync_routes.rs          every route in the app is registered here
  controllers/               thin HTTP layer: parse request, call a model fn, shape the response
  models/                    all SQL lives here (sqlx::query!/query_as!)
  datamodels/                request/response structs (serde) + structs that mirror FPL's own API shapes
  helpers/                    stateless logic that isn't a DB call
migrations/                 sqlx migrations, applied in filename-timestamp order
```

The layering is consistent everywhere: **controller** (actix handler, extracts `AuthUser`/`web::Json`/`web::Path`, calls a model function, wraps the result in `response_helper::success`/`failure`) → **model** (owns the SQL for one feature area) → **helper** (pure functions with no I/O — validation rules, scoring math).

### `controllers/`
| File | Owns |
|---|---|
| `auth_controller.rs` | `/auth/nonce`, `/auth/verify` — the wallet login handshake |
| `catalog_controller.rs` | `/teams`, `/players` — read-only listings for the frontend's squad picker |
| `fantasy_team_controllers.rs` | `/fantasy/create-team`, `/fantasy/my-team` — the season-long Campaign squad |
| `fantasy_league_controller.rs` | `/fantasy/create-league`, `/fantasy/leagues/{id}/join` |
| `live_sync_controller.rs` | `/sync/live` — pulls live FPL data, also called by the background loop |
| `offcial_fpl_controllers.rs` | `/sync/teams`, `/sync/players` — one-time/season data sync (note: `offcial` is a real typo in the filename, left as-is rather than risk a rename) |
| `points_controller.rs` | `/fantasy/my-points`, `/fantasy/leagues/{id}/leaderboard` |
| `transfers_controller.rs` | `/fantasy/transfers` |

### `models/`
One file per feature area, named to match its controller (`fantasy_team_model.rs`, `points_engine_model.rs`, `transfers_model.rs`, `fantasy_league_model.rs`, `live_sync_model.rs`, `catalog_model.rs`, `auth_model.rs`, `official_fpl_sync_model.rs`). If you're looking for a specific query, it's in the model file named after the feature, not scattered.

### `helpers/`
Pure logic, no database access:
- `squad_rules.rs` — squad validation for Campaign (`validate_squad`) and Derby (`validate_derby_squad`): position quotas, budget cap, max-per-club, starting-XI shape
- `scoring_rules.rs` — `calculate_points()`, a pure function implementing current FPL scoring (goals/assists by position, clean sheets, cards, bonus, defensive contribution threshold)
- `fpl_meta.rs` — `current_gameweek()`, fetched from FPL's own bootstrap-static `is_current` flag
- `jwt.rs` — reads the signing secret from `JWT_SECRET` (never hardcoded — see [Auth](#auth--wallet-login))
- `auth_middleware.rs` — the `AuthUser` extractor (see below)
- `http_client.rs` — thin wrapper around `reqwest` for calling the FPL API
- `response_helper.rs` — the `{ success, status_code, message, data }` envelope every endpoint returns
- `db.rs` — connection pool setup
- `fantasy_league_helper.rs` — join-code generation

## Request lifecycle

1. `routes/sync_routes.rs::init` registers every route against a `web::ServiceConfig`.
2. A request hits an actix handler in `controllers/`. If the route needs auth, the handler takes an `AuthUser` parameter — actix calls `AuthUser::from_request` ([`auth_middleware.rs`](../src/helpers/auth_middleware.rs)) before the handler body runs, which reads the `Authorization: Bearer <jwt>` header, decodes it, and looks up the user by wallet address.
3. The controller calls into `models::*`, which runs the actual SQL and returns a `Result<T>` (`anyhow::Result`, not a custom error enum — errors are stringified into the JSON `message` field).
4. The controller wraps the result via `response_helper::success`/`failure` and returns an `HttpResponse`.

Every response has the same shape:
```json
{ "success": true, "status_code": 200, "message": "...", "data": { ... } }
```

## Auth — wallet login

No passwords. The flow (`auth_controller.rs`):

1. `POST /auth/nonce { wallet }` — generates a random nonce, stores it on the `users` row for that wallet address.
2. Client signs the nonce with their Solana wallet (`ed25519`, detached signature).
3. `POST /auth/verify { wallet, signature }` — decodes the wallet's base58 public key, verifies the signature against the stored nonce with `ed25519-dalek`, clears the nonce (one-time use), and issues a JWT (`sub: wallet`, 24h expiry) signed with `JWT_SECRET`.
4. Every subsequent request sends that JWT as a bearer token; `auth_middleware.rs` decodes it and re-resolves the user by wallet address on every request (no session store).

**`DEV_MODE`**: if `DEV_MODE=true`, the `AuthUser` extractor skips all of the above and returns a fixed user (`DEV_USER_WALLET`) regardless of what's in the `Authorization` header. This must never be true outside local development — it's a full auth bypass. `docker-compose.yml` defaults it to `false`.

## Data sync — where player/match data comes from

FPLX doesn't have its own scouts — it polls the official Fantasy Premier League API (`fantasy.premierleague.com/api/...`), which is free, unauthenticated, and has no published rate limit but no true push/websocket either. Three endpoints matter:

- `bootstrap-static/` — full team + player list, season-aggregate stats, and the current gameweek (`is_current` flag). Backs `sync/teams`, `sync/players`, and `fpl_meta::current_gameweek()`.
- `fixtures/?event={gw}` — one gameweek's fixtures: scores, kickoff time, `started`/`finished`. Backs the `fixtures` table.
- `event/{gw}/live/` — per-player minutes/goals/assists/bonus/BPS for the current gameweek, with an `explain` array mapping each stat line to a specific fixture. This is what `live_sync_model::upsert_live_stats` writes into `player_match_stats`.

**The live loop** (`main.rs::run_live_sync_loop`): spawned once at startup via `tokio::spawn`, calls `live_sync_controller::sync_live_gameweek` in a loop — 60s between polls if any fixture is currently live, 15 minutes otherwise. `POST /sync/live` triggers the same function manually.

Every live-sync tick also calls `points_engine_model::recalculate_gameweek` — raw stats and fantasy points are always kept in sync in the same request.

## Points engine

`scoring_rules::calculate_points` is a pure function over one player's one-fixture stat line — it doesn't touch the database. `points_engine_model::recalculate_gameweek(pool, gameweek)` is what actually runs it: pulls every `player_match_stats` row for that gameweek (joined with the player's position and the fixture's score, to compute clean sheets / goals conceded), calls `calculate_points`, writes the result back to `player_match_stats.points`, then rolls each player's gameweek total into `players.points`.

Two things worth knowing if you're reading this code:

- **`players.points` is a cache, not history.** It only ever holds whichever gameweek was last recalculated for that player. Anything that needs a *specific* gameweek's score — Round contests, Derby contests — must read `player_match_stats.points` directly (joined to `fixtures` for the gameweek/fixture filter), never `players.points`. `points_engine_model::get_round_entry_points` / `get_derby_entry_points` do this correctly; `get_team_points` (Campaign) intentionally uses the cache because it's always asking about the *current* gameweek.
- **Captain doubling has a fallback.** If the captain logged 0 minutes in the relevant gameweek/fixture, the vice-captain's points are doubled instead — matches real FPL behavior. This logic is duplicated (deliberately, not accidentally) across `get_team_points`, `get_round_entry_points`, and `get_derby_entry_points`, because each reads minutes from a different scope (current gameweek vs. a pinned gameweek vs. a single fixture).

## Squad building & the three contest types

`fantasy_leagues.contest_type` is one of three values, each with a completely different scoring/entry model:

| `contest_type` | Squad lives in | Rules (`squad_rules.rs`) | Scored from |
|---|---|---|---|
| `campaign` | `fantasy_teams` / `fantasy_team_players` (relational, persistent, supports transfers) | `validate_squad`: 15 players, 2/5/5/3 GK/DEF/MID/FWD, 100.0m budget, max 3/club, 11-man starting XI | `players.points` (current gameweek cache) |
| `round` | `fantasy_league_participants.team_data` (JSON snapshot, submitted once) | Same as Campaign — reuses `validate_squad` | `player_match_stats` filtered to the league's pinned `gameweek` |
| `derby` | `fantasy_league_participants.team_data` (JSON snapshot) | `validate_derby_squad`: 11 players, no bench, drawn only from the fixture's two clubs, max 7/club | `player_match_stats` filtered to the league's pinned `fixture_id` |

Campaign is the only format with persistent, editable state — Derby and Round are one-shot entries validated and locked at submission time (`fantasy_league_model::join_league` branches on `contest_type` and does the validation inline; there's no shared "contest" abstraction beyond the `fantasy_leagues` row itself).

**Locking**: transfers ([`transfers_model::is_gameweek_locked`](../src/models/transfers_model.rs)) and Derby/Round joins both gate on `fixtures.started` for the relevant scope — once a fixture has kicked off, that door closes. There is no per-gameweek squad snapshot system, so this is a hard on/off gate, not a "your old squad still applies to last week" mechanism.

## Transfers

`POST /fantasy/transfers` takes a batch of same-position swaps. Each team banks 1 free transfer per gameweek (lazily replenished — `transfers_model::make_transfers` computes how many gameweeks have passed since `fantasy_teams.last_transfer_gameweek` and grants that many, capped at 5). Anything beyond the free ones costs 4 points, logged per-swap in `fantasy_transfers` and subtracted from the team's total in `points_engine_model::get_team_points` (`gross_points - points_hit = total_points`).

You cannot transfer out your current captain or vice-captain — there's no endpoint yet to reassign the armband, so this would otherwise be a dead end.

## Database

Postgres, no ORM. Key tables: `users`, `teams`, `players`, `fixtures`, `player_match_stats`, `fantasy_teams`, `fantasy_team_players`, `fantasy_leagues`, `fantasy_league_participants`, `fantasy_transfers`, `league_payouts` (unused so far — reserved for on-chain payout tracking). `migrations/schema.sql` is a hand-maintained full reference dump of the current schema — useful for reading, but **migrations are the actual source of truth**, not this file.

### Migration history has a wrinkle — read this before adding a new one

The very first migration (`20260131190253_init_schema.sql`) only creates `teams`/`players`/`fixtures`/`player_match_stats`, and does so with a couple of wrong column types (`player_match_stats.id` with no auto-increment default, `players.form` as `TEXT`). Every other table — `users`, `fantasy_teams`, `fantasy_leagues`, etc. — only ever existed because `schema.sql` was run by hand against the live database once, long before migrations were taken seriously. None of that was ever captured as a migration, so a genuinely fresh database (a new environment, CI, Docker) was missing most of the schema and had two columns of the wrong type.

This was found (not guessed at) by actually building the Docker image and running it against a brand-new Postgres volume — it crashed in three different ways, one at a time, each fixed with its own migration:
- `20260912000000_add_core_league_tables.sql` — backfills every table the init migration never created, using `CREATE TABLE IF NOT EXISTS` (a no-op on a database that already has them). This migration's timestamp deliberately sits *before* several already-applied migrations, so `main.rs` runs `sqlx::migrate!(...).set_ignore_missing(true)` — without that flag, sqlx refuses to apply an "old" migration once newer ones are already recorded.
- `20260912185241_fix_player_match_stats_id_sequence.sql` — adds the missing auto-increment sequence.
- `20260912193754_fix_players_form_column_type.sql` — converts `form` from `TEXT` to `REAL`, guarded by an `information_schema` check so it's a no-op on a database where the column is already correct.

**The lesson for future migrations**: this codebase's migration history is not fully trustworthy as a description of what the *first* migration actually produces. If you add a new migration, test it against a wiped `docker-compose` volume (`docker compose down -v && docker compose up --build`), not just the long-lived local dev database — the local DB will hide problems like this because it was never actually built from these migrations in the first place.

## Running locally

```bash
cp .env.example .env     # fill in DATABASE_URL, JWT_SECRET at minimum
cargo run                 # runs on :8080 (or $PORT), auto-applies migrations
```

Useful env vars (`.env.example` has the full list): `DATABASE_URL`, `JWT_SECRET` (required, no default), `DEV_MODE`/`DEV_USER_WALLET` (local auth bypass, never in prod), `CORS_ORIGINS` (comma-separated, defaults to `http://localhost:3000`), `PORT` (defaults to 8080).

After changing a query or adding a migration, regenerate the offline sqlx cache (committed to the repo, used for `SQLX_OFFLINE=true` Docker builds that never touch a live DB):
```bash
cargo sqlx migrate run          # apply locally
cargo sqlx prepare              # regenerate .sqlx/*.json
```

## Docker & deployment

`Dockerfile` is a two-stage build (rustls only — `reqwest`'s default OpenSSL/`native-tls` dependency was deliberately dropped via `default-features = false`, which is also why the image needs no `libssl-dev`). `docker-compose.yml` runs `postgres` + `app` + the Next.js `web` service together, with the app's healthcheck hitting `GET /health` (a DB ping). Migrations run automatically on container start — there is no separate migrate step to remember.

```bash
cp .env.example .env      # set JWT_SECRET, POSTGRES_PASSWORD at minimum
docker compose up --build
```

## API reference

All routes are prefixed as shown; `AuthUser` = requires `Authorization: Bearer <jwt>`.

| Method | Path | Auth | Purpose |
|---|---|---|---|
| GET | `/health` | – | DB-ping healthcheck |
| GET | `/teams` | – | List real-world clubs |
| GET | `/players` | – | List all players (for squad building) |
| POST | `/sync/teams` | – | Pull clubs from FPL |
| POST | `/sync/players` | – | Pull season player stats from FPL |
| POST | `/sync/live` | – | Pull live fixtures/stats + recalc points (also runs on a background timer) |
| POST | `/auth/nonce` | – | Step 1 of wallet login |
| POST | `/auth/verify` | – | Step 2 of wallet login → JWT |
| POST | `/fantasy/create-team` | ✓ | Create your Campaign squad |
| GET | `/fantasy/my-team` | ✓ | View your Campaign squad |
| POST | `/fantasy/create-league` | ✓ | Create a Campaign/Derby/Round league |
| POST | `/fantasy/leagues/{id}/join` | ✓ | Join a league (entry required for Derby/Round) |
| GET | `/fantasy/leagues/{id}/leaderboard` | – | League standings |
| GET | `/fantasy/my-points` | ✓ | Your Campaign squad's current-gameweek points |
| POST | `/fantasy/transfers` | ✓ | Submit a batch of Campaign transfers |

## Known gaps (intentional, not oversights)

- No endpoint to change captain/vice-captain independently of a transfer
- No "list my leagues" endpoint
- Only the starting XI counts toward Campaign points — no auto-substitution if a starter blanks (`bench_order` is stored but unused)
- Public/private league creation limits (3 public / 2 private, enforced by a DB trigger) apply per-creator across *all* contest types combined, not per type
- Nothing here is on-chain yet — wallet auth is real, but `entry_fee`/`league_payouts` are plain database columns, not an escrow program
