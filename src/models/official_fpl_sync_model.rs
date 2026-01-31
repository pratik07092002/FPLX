use sqlx::PgPool;
use anyhow::Result;
use crate::datamodels::official_fpl_models::{
    FplTeam,
    FPLPlayer
};


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




pub async fn upsert_players(
    pool: &PgPool,
    players: &[FPLPlayer],
) -> Result<()> {

    for player in players {

        sqlx::query!(
            r#"
            INSERT INTO players (
                id,
                first_name,
                second_name,
                photo,
                team_id,

                form,
                points,
                total_points,
                minutes_played,

                goals_scored,
                assists,
                yellow_cards,
                red_cards,
                saves,
                starts,

                news, 
                position
            )
            VALUES (
                $1,$2,$3,$4,$5,
                $6,$7,$8,$9,
                $10,$11,$12,$13,$14,$15,
                $16,$17
            )

            ON CONFLICT (id)
            DO UPDATE SET

                first_name = EXCLUDED.first_name,
                second_name = EXCLUDED.second_name,
                photo = EXCLUDED.photo,
                team_id = EXCLUDED.team_id,

                form = EXCLUDED.form,
                points = EXCLUDED.points,
                total_points = EXCLUDED.total_points,
                minutes_played = EXCLUDED.minutes_played,

                goals_scored = EXCLUDED.goals_scored,
                assists = EXCLUDED.assists,
                yellow_cards = EXCLUDED.yellow_cards,
                red_cards = EXCLUDED.red_cards,
                saves = EXCLUDED.saves,
                starts = EXCLUDED.starts,

                news = EXCLUDED.news,
                position = EXCLUDED.position
            "#,

            // Primary
            player.id,
            player.first_name,
            player.second_name,
            player.photo,
            player.team,

            // Stats
            player.form,
            player.points,
            player.total_points,
            player.minutes,

            player.goals_scored,
            player.assists,
            player.yellow_cards,
            player.red_cards,
            player.saves,
            player.starts,

            player.news,
            player.position
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
