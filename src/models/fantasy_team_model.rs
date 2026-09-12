use anyhow::Result;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;
use crate::datamodels::official_fpl_models::FPLPlayer;
use crate::datamodels::fantasy_team_data_models::{MyTeamResponse, SquadPlayerView};
use crate::helpers::squad_rules::SquadPlayerInfo;

pub async fn get_squad_player_info(
    pool: &PgPool,
    player_ids: &[i32],
) -> Result<Vec<SquadPlayerInfo>> {

    let rows = sqlx::query!(
        r#"
        SELECT id, position, team_id, now_cost
        FROM players
        WHERE id = ANY($1)
        "#,
        player_ids
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SquadPlayerInfo {
            id: r.id,
            position: r.position,
            team_id: r.team_id,
            now_cost: r.now_cost,
        })
        .collect())
}


pub async fn create_team(
    tx: &mut Transaction<'_, Postgres>,

    user_id: Uuid,

    captain_id: i32,

    vice_captain_id: i32,

    current_gameweek: i32,
) -> Result<Uuid> {

    let rec = sqlx::query!(
        r#"
        INSERT INTO fantasy_teams
        (user_id, captain_id, vice_captain_id, last_transfer_gameweek)

        VALUES ($1,$2,$3,$4)

        RETURNING id
        "#,
        user_id,
        captain_id,
        vice_captain_id,
        current_gameweek,
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(rec.id)
}



pub async fn insert_players(
    tx: &mut Transaction<'_, Postgres>,

    team_id: Uuid,

    players: &[i32],

    starting_ids: &HashSet<i32>,
) -> Result<()> {

    let mut bench_order: i16 = 0;

    for pid in players {

        let is_starting = starting_ids.contains(pid);

        let this_bench_order = if is_starting {
            None
        } else {
            let order = bench_order;
            bench_order += 1;
            Some(order)
        };

        sqlx::query!(
            r#"
            INSERT INTO fantasy_team_players
            (fantasy_team_id, player_id, is_starting, bench_order)

            VALUES ($1,$2,$3,$4)
            "#,
            team_id,
            pid,
            is_starting,
            this_bench_order,
        )
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}


pub async fn mark_team_created(
    tx: &mut Transaction<'_, Postgres>,

    user_id: Uuid,
) -> Result<()> {

    sqlx::query!(
        r#"
        UPDATE users
        SET is_team_created = true
        WHERE id = $1
        "#,
        user_id
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}




pub async fn get_my_team(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<MyTeamResponse> {

    // Get team
    let team = sqlx::query!(
        r#"
        SELECT id, captain_id, vice_captain_id
        FROM fantasy_teams
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    // Get all players, with their starting/bench status on this team
    let rows = sqlx::query!(
        r#"
        SELECT
            p.id,
            p.first_name,
            p.second_name,
            p.photo,
            p.team_id AS team,
            p.form,
            p.points,
            p.total_points,
            p.minutes_played AS minutes,
            p.goals_scored,
            p.assists,
            p.yellow_cards,
            p.red_cards,
            p.saves,
            p.starts,
            p.news,
            p.position,
            p.now_cost,
            fp.is_starting,
            fp.bench_order
        FROM fantasy_team_players fp
        JOIN players p ON p.id = fp.player_id
        WHERE fp.fantasy_team_id = $1
        ORDER BY fp.is_starting DESC, fp.bench_order ASC NULLS FIRST
        "#,
        team.id
    )
    .fetch_all(pool)
    .await?;

    let players: Vec<SquadPlayerView> = rows
        .into_iter()
        .map(|r| SquadPlayerView {
            player: FPLPlayer {
                id: r.id,
                first_name: Some(r.first_name),
                second_name: Some(r.second_name),
                photo: r.photo,
                team: Some(r.team),
                form: r.form,
                points: r.points,
                total_points: r.total_points,
                minutes: r.minutes,
                goals_scored: r.goals_scored,
                assists: r.assists,
                yellow_cards: r.yellow_cards,
                red_cards: r.red_cards,
                saves: r.saves,
                starts: r.starts,
                news: r.news,
                position: Some(r.position),
                now_cost: Some(r.now_cost),
            },
            is_starting: r.is_starting,
            bench_order: r.bench_order,
        })
        .collect();

    // Find captain
    let captain = players
        .iter()
        .find(|p| p.player.id == team.captain_id)
        .map(|p| p.player.clone())
        .ok_or_else(|| anyhow::anyhow!("Captain not found"))?;

    // Find VC
    let vice_captain = players
        .iter()
        .find(|p| p.player.id == team.vice_captain_id)
        .map(|p| p.player.clone())
        .ok_or_else(|| anyhow::anyhow!("Vice captain not found"))?;

    Ok(MyTeamResponse {
        team_id: team.id,
        captain,
        vice_captain,
        players,
    })
}
