use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;
use anyhow::Result;

use crate::datamodels::fantasy_league_data_model::{
CreateLeagueRequest,
FantasyLeague,
LeagueParticipant,
LeagueResponse
};

pub async fn create_league(
    pool: &PgPool,
    name: &str,
    league_type: &str,
    user_id: Uuid,
    user_name: &str,
    join_code: Option<String>,
) -> Result<LeagueResponse> {

    let rec = sqlx::query!(
        r#"
        INSERT INTO fantasy_leagues
        (league_name, created_by_user_id, created_by_user_name,
         league_type, join_code)

        VALUES ($1,$2,$3,$4,$5)

        RETURNING id, league_name, league_type, join_code
        "#,
        name,
        user_id,
        user_name,
        league_type,
        join_code
    )
    .fetch_one(pool)
    .await?;

    Ok(LeagueResponse {
        id: rec.id,
        league_name: rec.league_name,
        league_type: rec.league_type,
        join_code: rec.join_code,
    })
}




pub async fn add_participant(
    pool: &PgPool,
    league_id: i32,
    user_id: Uuid,
    user_name: &str,
) -> Result<()> {

    sqlx::query!(
        r#"
        INSERT INTO fantasy_league_participants
        (league_id, user_id, user_name, team_data)

        VALUES ($1,$2,$3,'{}'::jsonb)
        "#,
        league_id,
        user_id,
        user_name,
    )
    .execute(pool)
    .await?;

    Ok(())
}





pub async fn create_league_tx(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    league_type: &str,
    user_id: Uuid,
    user_name: &str,
    join_code: Option<String>,
) -> Result<crate::datamodels::fantasy_league_data_model::LeagueResponse> {

    let rec = sqlx::query!(
        r#"
        INSERT INTO fantasy_leagues
        (league_name, created_by_user_id, created_by_user_name,
         league_type, join_code)

        VALUES ($1,$2,$3,$4,$5)

        RETURNING id, league_name, league_type, join_code
        "#,
        name,
        user_id,
        user_name,
        league_type,
        join_code
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(crate::datamodels::fantasy_league_data_model::LeagueResponse {
        id: rec.id,
        league_name: rec.league_name,
        league_type: rec.league_type,
        join_code: rec.join_code,
    })
}

pub async fn add_participant_tx(
    tx: &mut Transaction<'_, Postgres>,
    league_id: i32,
    user_id: Uuid,
    user_name: &str,
) -> Result<()> {

    sqlx::query!(
        r#"
        INSERT INTO fantasy_league_participants
        (league_id, user_id, user_name, team_data)

        VALUES ($1,$2,$3,'{}'::jsonb)
        "#,
        league_id,
        user_id,
        user_name
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
