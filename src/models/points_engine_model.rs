use anyhow::Result;
use sqlx::types::Json;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::datamodels::fantasy_league_data_model::ContestEntry;
use crate::datamodels::points_data_model::{LeaderboardEntry, PlayerPoints, TeamPoints};
use crate::helpers::scoring_rules::{self, FixtureStatLine};
use crate::models::fantasy_league_model;

/// Recomputes fantasy points for every stat line recorded in a gameweek, and
/// refreshes each affected player's current-gameweek point total. Safe to call
/// repeatedly as new live stats come in — every call fully replaces the prior
/// values for that gameweek rather than accumulating on top of them.
pub async fn recalculate_gameweek(pool: &PgPool, gameweek: i32) -> Result<usize> {
    let rows = sqlx::query!(
        r#"
        SELECT
            pms.id,
            p.position,
            pms.team_side,
            pms.minutes,
            pms.goals,
            pms.assists,
            pms.own_goals,
            pms.penalties_saved,
            pms.penalties_missed,
            pms.yellow_cards,
            pms.red_cards,
            pms.saves,
            pms.bonus,
            pms.defensive_contribution,
            f.home_score,
            f.away_score
        FROM player_match_stats pms
        JOIN players p ON p.id = pms.player_id
        JOIN fixtures f ON f.id = pms.fixture_id
        WHERE f.gameweek = $1
        "#,
        gameweek
    )
    .fetch_all(pool)
    .await?;

    let mut updated = 0;

    for row in rows {
        let goals_conceded = if row.team_side.as_deref() == Some("h") {
            row.away_score.unwrap_or(0)
        } else {
            row.home_score.unwrap_or(0)
        };

        let points = scoring_rules::calculate_points(&FixtureStatLine {
            position: row.position,
            minutes: row.minutes.unwrap_or(0),
            goals: row.goals.unwrap_or(0),
            assists: row.assists.unwrap_or(0),
            own_goals: row.own_goals.unwrap_or(0),
            penalties_saved: row.penalties_saved.unwrap_or(0),
            penalties_missed: row.penalties_missed.unwrap_or(0),
            yellow_cards: row.yellow_cards.unwrap_or(0),
            red_cards: row.red_cards.unwrap_or(0),
            saves: row.saves.unwrap_or(0),
            bonus: row.bonus.unwrap_or(0),
            defensive_contribution: row.defensive_contribution.unwrap_or(0),
            goals_conceded,
        });

        sqlx::query!(
            "UPDATE player_match_stats SET points = $1 WHERE id = $2",
            points,
            row.id
        )
        .execute(pool)
        .await?;

        updated += 1;
    }

    sqlx::query!(
        r#"
        UPDATE players p
        SET points = sub.total
        FROM (
            SELECT pms.player_id, SUM(pms.points) AS total
            FROM player_match_stats pms
            JOIN fixtures f ON f.id = pms.fixture_id
            WHERE f.gameweek = $1
            GROUP BY pms.player_id
        ) sub
        WHERE p.id = sub.player_id
        "#,
        gameweek
    )
    .execute(pool)
    .await?;

    Ok(updated)
}

async fn played_minutes(pool: &PgPool, player_id: i32, gameweek: i32) -> Result<i64> {
    let rec = sqlx::query!(
        r#"
        SELECT COALESCE(SUM(pms.minutes), 0) AS "minutes!"
        FROM player_match_stats pms
        JOIN fixtures f ON f.id = pms.fixture_id
        WHERE f.gameweek = $1 AND pms.player_id = $2
        "#,
        gameweek,
        player_id
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.minutes)
}

/// A user's 15-player squad with current-gameweek points applied, captain
/// doubled — falling back to the vice-captain if the captain didn't play.
pub async fn get_team_points(pool: &PgPool, user_id: Uuid, gameweek: i32) -> Result<TeamPoints> {
    let team = sqlx::query!(
        "SELECT id, captain_id, vice_captain_id FROM fantasy_teams WHERE user_id = $1",
        user_id
    )
    .fetch_one(pool)
    .await?;

    let players = sqlx::query!(
        r#"
        SELECT p.id, p.first_name, p.second_name, p.position, p.points, fp.is_starting
        FROM fantasy_team_players fp
        JOIN players p ON p.id = fp.player_id
        WHERE fp.fantasy_team_id = $1
        "#,
        team.id
    )
    .fetch_all(pool)
    .await?;

    let captain_minutes = played_minutes(pool, team.captain_id, gameweek).await?;
    let vice_minutes = played_minutes(pool, team.vice_captain_id, gameweek).await?;

    let mut total = 0;
    let mut out = Vec::with_capacity(players.len());

    for p in players {
        let points = p.points.unwrap_or(0);

        // Only the starting XI counts toward the team total — the bench is
        // informational only until auto-substitutions are implemented.
        if p.is_starting {
            total += points;
        }

        out.push(PlayerPoints {
            player_id: p.id,
            first_name: p.first_name,
            second_name: p.second_name,
            position: p.position,
            points,
            is_captain: p.id == team.captain_id,
            is_vice_captain: p.id == team.vice_captain_id,
            is_starting: p.is_starting,
        });
    }

    // Captain/vice-captain are guaranteed to be starters by squad validation.
    if captain_minutes > 0 {
        total += out.iter().find(|p| p.is_captain).map(|p| p.points).unwrap_or(0);
    } else if vice_minutes > 0 {
        total += out.iter().find(|p| p.is_vice_captain).map(|p| p.points).unwrap_or(0);
    }

    let points_hit = crate::models::transfers_model::get_points_hit(pool, team.id, gameweek).await?;

    Ok(TeamPoints {
        team_id: team.id,
        gameweek,
        gross_points: total,
        points_hit,
        total_points: total - points_hit,
        players: out,
    })
}

/// Points for a one-shot Round entry — scored from the specific gameweek
/// the league is pinned to, never from the mutable `players.points` cache
/// (which only ever holds whatever gameweek was last recalculated).
pub async fn get_round_entry_points(pool: &PgPool, entry: &ContestEntry, gameweek: i32) -> Result<i32> {
    let rows = sqlx::query!(
        r#"
        SELECT pms.player_id, COALESCE(SUM(pms.points), 0)::INTEGER AS "points!"
        FROM player_match_stats pms
        JOIN fixtures f ON f.id = pms.fixture_id
        WHERE f.gameweek = $1 AND pms.player_id = ANY($2)
        GROUP BY pms.player_id
        "#,
        gameweek,
        &entry.starting_ids,
    )
    .fetch_all(pool)
    .await?;

    let points_by: HashMap<i32, i32> = rows.into_iter().map(|r| (r.player_id, r.points)).collect();

    let mut total: i32 = entry
        .starting_ids
        .iter()
        .map(|id| points_by.get(id).copied().unwrap_or(0))
        .sum();

    let captain_minutes = played_minutes(pool, entry.captain_id, gameweek).await?;
    let vice_minutes = played_minutes(pool, entry.vice_captain_id, gameweek).await?;

    if captain_minutes > 0 {
        total += points_by.get(&entry.captain_id).copied().unwrap_or(0);
    } else if vice_minutes > 0 {
        total += points_by.get(&entry.vice_captain_id).copied().unwrap_or(0);
    }

    Ok(total)
}

/// Points for a one-shot Derby entry — scored from that single fixture only.
pub async fn get_derby_entry_points(pool: &PgPool, entry: &ContestEntry, fixture_id: i32) -> Result<i32> {
    let rows = sqlx::query!(
        r#"
        SELECT player_id, points, minutes
        FROM player_match_stats
        WHERE fixture_id = $1 AND player_id = ANY($2)
        "#,
        fixture_id,
        &entry.players,
    )
    .fetch_all(pool)
    .await?;

    let mut points_by: HashMap<i32, i32> = HashMap::new();
    let mut minutes_by: HashMap<i32, i32> = HashMap::new();
    for r in rows {
        points_by.insert(r.player_id, r.points);
        minutes_by.insert(r.player_id, r.minutes.unwrap_or(0));
    }

    let mut total: i32 = entry
        .players
        .iter()
        .map(|id| points_by.get(id).copied().unwrap_or(0))
        .sum();

    let captain_minutes = minutes_by.get(&entry.captain_id).copied().unwrap_or(0);
    let vice_minutes = minutes_by.get(&entry.vice_captain_id).copied().unwrap_or(0);

    if captain_minutes > 0 {
        total += points_by.get(&entry.captain_id).copied().unwrap_or(0);
    } else if vice_minutes > 0 {
        total += points_by.get(&entry.vice_captain_id).copied().unwrap_or(0);
    }

    Ok(total)
}

pub async fn get_league_leaderboard(pool: &PgPool, league_id: i32) -> Result<Vec<LeaderboardEntry>> {
    let league = fantasy_league_model::get_league(pool, league_id).await?;

    let mut entries = Vec::new();

    match league.contest_type.as_str() {
        "campaign" => {
            let gameweek = crate::helpers::fpl_meta::current_gameweek().await?;

            let participants = sqlx::query!(
                "SELECT user_id, user_name FROM fantasy_league_participants WHERE league_id = $1",
                league_id
            )
            .fetch_all(pool)
            .await?;

            for p in participants {
                let total_points = get_team_points(pool, p.user_id, gameweek)
                    .await
                    .map(|tp| tp.total_points)
                    .unwrap_or(0); // no squad created yet

                entries.push(LeaderboardEntry {
                    user_id: p.user_id,
                    user_name: p.user_name,
                    total_points,
                });
            }
        }

        "round" | "derby" => {
            let participants = sqlx::query!(
                r#"
                SELECT user_id, user_name, team_data AS "team_data!: Json<ContestEntry>"
                FROM fantasy_league_participants
                WHERE league_id = $1
                "#,
                league_id
            )
            .fetch_all(pool)
            .await?;

            for p in participants {
                let total_points = if league.contest_type == "round" {
                    get_round_entry_points(pool, &p.team_data.0, league.gameweek.unwrap_or(0)).await?
                } else {
                    get_derby_entry_points(pool, &p.team_data.0, league.fixture_id.unwrap_or(0)).await?
                };

                entries.push(LeaderboardEntry {
                    user_id: p.user_id,
                    user_name: p.user_name,
                    total_points,
                });
            }
        }

        _ => {}
    }

    entries.sort_by(|a, b| b.total_points.cmp(&a.total_points));

    Ok(entries)
}
