use anyhow::Result;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use crate::datamodels::official_fpl_models::FPLPlayer;
use crate::datamodels::fantasy_team_data_models::MyTeamResponse;

pub async fn create_team(
    tx: &mut Transaction<'_, Postgres>,

    user_id: Uuid,

    captain_id: i32,

    vice_captain_id: i32,
) -> Result<Uuid> {

    let rec = sqlx::query!(
        r#"
        INSERT INTO fantasy_teams
        (user_id, captain_id, vice_captain_id)

        VALUES ($1,$2,$3)

        RETURNING id
        "#,
        user_id,
        captain_id,
        vice_captain_id
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(rec.id)
}



pub async fn insert_players(
    tx: &mut Transaction<'_, Postgres>,

    team_id: Uuid,

    players: &[i32],
) -> Result<()> {

    for pid in players {

        sqlx::query!(
            r#"
            INSERT INTO fantasy_team_players
            (fantasy_team_id, player_id)

            VALUES ($1,$2)
            "#,
            team_id,
            pid
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

    // Get all players
    let players = sqlx::query_as!(
        FPLPlayer,
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
            p.position
        FROM fantasy_team_players fp
        JOIN players p ON p.id = fp.player_id
        WHERE fp.fantasy_team_id = $1
        "#,
        team.id
    )
    .fetch_all(pool)
    .await?;

    // Find captain
    let captain = players
        .iter()
        .find(|p| p.id == team.captain_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Captain not found"))?;

    // Find VC
    let vice_captain = players
        .iter()
        .find(|p| p.id == team.vice_captain_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Vice captain not found"))?;

    Ok(MyTeamResponse {
        team_id: team.id,
        captain,
        vice_captain,
        players,
    })
}
