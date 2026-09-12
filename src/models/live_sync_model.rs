use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::datamodels::official_fpl_models::{FplFixture, FplLiveElement};

pub async fn upsert_fixtures(
    pool: &PgPool,
    gameweek: i32,
    fixtures: &[FplFixture],
) -> Result<()> {
    for f in fixtures {
        sqlx::query!(
            r#"
            INSERT INTO fixtures (
                id, gameweek, start_time,
                home_team_id, away_team_id,
                started, finished,
                home_score, away_score
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)

            ON CONFLICT (id)
            DO UPDATE SET
                start_time = EXCLUDED.start_time,
                started = EXCLUDED.started,
                finished = EXCLUDED.finished,
                home_score = EXCLUDED.home_score,
                away_score = EXCLUDED.away_score
            "#,
            f.id,
            gameweek,
            f.kickoff_time.map(|d| d.naive_utc()),
            f.team_h,
            f.team_a,
            f.started.unwrap_or(false),
            f.finished.unwrap_or(false),
            f.team_h_score,
            f.team_a_score,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_player_team_map(pool: &PgPool) -> Result<HashMap<i32, i32>> {
    let rows = sqlx::query!("SELECT id, team_id FROM players")
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| (r.id, r.team_id)).collect())
}

pub async fn upsert_live_stats(
    pool: &PgPool,
    elements: &[FplLiveElement],
    player_team: &HashMap<i32, i32>,
    fixture_sides: &HashMap<i32, (i32, i32)>,
) -> Result<()> {
    for el in elements {
        // Skip players who haven't been involved in this gameweek yet.
        let Some(fixture_id) = el.explain.first().map(|e| e.fixture) else {
            continue;
        };

        let Some(team_id) = player_team.get(&el.id) else {
            continue;
        };

        let Some((home_id, away_id)) = fixture_sides.get(&fixture_id) else {
            continue;
        };

        let team_side = if team_id == home_id {
            "h"
        } else if team_id == away_id {
            "a"
        } else {
            continue;
        };

        let s = &el.stats;

        sqlx::query!(
            r#"
            INSERT INTO player_match_stats (
                fixture_id, player_id, team_side,
                minutes, goals, assists, own_goals,
                penalties_saved, penalties_missed,
                yellow_cards, red_cards, saves,
                bonus, bps, defensive_contribution
            )
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)

            ON CONFLICT (fixture_id, player_id)
            DO UPDATE SET
                team_side = EXCLUDED.team_side,
                minutes = EXCLUDED.minutes,
                goals = EXCLUDED.goals,
                assists = EXCLUDED.assists,
                own_goals = EXCLUDED.own_goals,
                penalties_saved = EXCLUDED.penalties_saved,
                penalties_missed = EXCLUDED.penalties_missed,
                yellow_cards = EXCLUDED.yellow_cards,
                red_cards = EXCLUDED.red_cards,
                saves = EXCLUDED.saves,
                bonus = EXCLUDED.bonus,
                bps = EXCLUDED.bps,
                defensive_contribution = EXCLUDED.defensive_contribution
            "#,
            fixture_id,
            el.id,
            team_side,
            s.minutes,
            s.goals_scored,
            s.assists,
            s.own_goals,
            s.penalties_saved,
            s.penalties_missed,
            s.yellow_cards,
            s.red_cards,
            s.saves,
            s.bonus,
            s.bps,
            s.defensive_contribution,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
