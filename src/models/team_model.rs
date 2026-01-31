use sqlx::PgPool;
use anyhow::Result;
use crate::datamodels::official_fpl_models::FplTeam;


pub async fn upsert_teams(
    pool: &PgPool,
    teams: &[FplTeam]
) -> Result<()> {
    for team in teams {
        sqlx::query!(
            r#"
            INSERT INTO teams (id, name , short_name , position) 
            VALUES ($1,$2,$3,$4) 
            ON CONFLICT (id) 
            DO UPDATE SET 
            name = EXCLUDED.name,
            short_name = EXCLUDED.short_name, 
            position = EXCLUDED.position
            "#,
            team.id,
            team.name,
            team.short_name,
            team.position
        ).execute(pool).await?;
    }

    Ok(())
}