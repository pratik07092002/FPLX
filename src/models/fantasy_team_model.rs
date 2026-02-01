use anyhow::Result;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

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
