use anyhow::Result;
use sqlx::PgPool;

use crate::datamodels::catalog_data_model::PlayerListItem;
use crate::datamodels::official_fpl_models::FplTeam;

pub async fn list_teams(pool: &PgPool) -> Result<Vec<FplTeam>> {
    let rows = sqlx::query_as!(
        FplTeam,
        r#"SELECT id, name, short_name, position FROM teams ORDER BY name"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn list_players(pool: &PgPool) -> Result<Vec<PlayerListItem>> {
    let rows = sqlx::query_as!(
        PlayerListItem,
        r#"
        SELECT
            p.id,
            p.first_name,
            p.second_name,
            p.photo,
            p.team_id,
            t.name AS team_name,
            t.short_name AS team_short_name,
            p.position,
            p.now_cost,
            p.form,
            p.total_points,
            p.news
        FROM players p
        JOIN teams t ON t.id = p.team_id
        ORDER BY p.position, p.now_cost DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}
