use anyhow::{bail, Result};
use sqlx::types::Json;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::datamodels::fantasy_league_data_model::{ContestEntry, LeagueResponse};
use crate::helpers::squad_rules;
use crate::models::{fantasy_team_model, transfers_model};

pub struct LeagueRow {
    pub id: i32,
    pub league_type: String,
    pub contest_type: String,
    pub fixture_id: Option<i32>,
    pub gameweek: Option<i32>,
    pub join_code: Option<String>,
}

pub async fn get_league(pool: &PgPool, league_id: i32) -> Result<LeagueRow> {
    let rec = sqlx::query!(
        r#"
        SELECT id, league_type, contest_type, fixture_id, gameweek, join_code
        FROM fantasy_leagues
        WHERE id = $1
        "#,
        league_id
    )
    .fetch_one(pool)
    .await?;

    Ok(LeagueRow {
        id: rec.id,
        league_type: rec.league_type,
        contest_type: rec.contest_type,
        fixture_id: rec.fixture_id,
        gameweek: rec.gameweek,
        join_code: rec.join_code,
    })
}

pub async fn create_league_tx(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    league_type: &str,
    contest_type: &str,
    fixture_id: Option<i32>,
    gameweek: Option<i32>,
    user_id: Uuid,
    user_name: &str,
    join_code: Option<String>,
) -> Result<LeagueResponse> {

    let rec = sqlx::query!(
        r#"
        INSERT INTO fantasy_leagues
        (league_name, created_by_user_id, created_by_user_name,
         league_type, contest_type, fixture_id, gameweek, join_code)

        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)

        RETURNING id, league_name, league_type, contest_type, fixture_id, gameweek, join_code
        "#,
        name,
        user_id,
        user_name,
        league_type,
        contest_type,
        fixture_id,
        gameweek,
        join_code
    )
    .fetch_one(&mut **tx)
    .await?;

    Ok(LeagueResponse {
        id: rec.id,
        league_name: rec.league_name,
        league_type: rec.league_type,
        contest_type: rec.contest_type,
        fixture_id: rec.fixture_id,
        gameweek: rec.gameweek,
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
        user_name,
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}

/// Joins a league — a Campaign join just needs the member's persistent
/// squad to already exist; Derby/Round joins submit their own one-shot
/// entry, validated against that contest's own rules and locked to the
/// specific fixture/gameweek it's scoped to.
pub async fn join_league(
    pool: &PgPool,
    league_id: i32,
    user_id: Uuid,
    user_name: &str,
    join_code_input: Option<&str>,
    entry: Option<ContestEntry>,
) -> Result<()> {
    let league = get_league(pool, league_id).await?;

    if league.league_type == "private" {
        let expected = league.join_code.as_deref();
        if expected.is_none() || join_code_input != expected {
            bail!("Invalid join code");
        }
    }

    let team_data = match league.contest_type.as_str() {
        "campaign" => {
            let is_team_created = sqlx::query!(
                "SELECT is_team_created FROM users WHERE id = $1",
                user_id
            )
            .fetch_one(pool)
            .await?
            .is_team_created;

            if !is_team_created {
                bail!("Create your squad before joining a campaign league");
            }

            serde_json::json!({})
        }

        "derby" => {
            let entry = entry.ok_or_else(|| anyhow::anyhow!("A derby entry is required to join"))?;
            let fixture_id = league
                .fixture_id
                .ok_or_else(|| anyhow::anyhow!("Derby league has no fixture configured"))?;

            let fixture = sqlx::query!(
                r#"SELECT home_team_id, away_team_id, started FROM fixtures WHERE id = $1"#,
                fixture_id
            )
            .fetch_one(pool)
            .await?;

            if fixture.started.unwrap_or(false) {
                bail!("This derby has already kicked off");
            }

            let home_team_id = fixture
                .home_team_id
                .ok_or_else(|| anyhow::anyhow!("Fixture has no home team set"))?;
            let away_team_id = fixture
                .away_team_id
                .ok_or_else(|| anyhow::anyhow!("Fixture has no away team set"))?;

            let squad_info = fantasy_team_model::get_squad_player_info(pool, &entry.players).await?;
            if squad_info.len() != entry.players.len() {
                bail!("One or more selected players do not exist");
            }

            squad_rules::validate_derby_squad(
                &squad_info,
                home_team_id,
                away_team_id,
                entry.captain_id,
                entry.vice_captain_id,
            )?;

            serde_json::to_value(&entry)?
        }

        "round" => {
            let entry = entry.ok_or_else(|| anyhow::anyhow!("A round entry is required to join"))?;
            let gameweek = league
                .gameweek
                .ok_or_else(|| anyhow::anyhow!("Round league has no gameweek configured"))?;

            if transfers_model::is_gameweek_locked(pool, gameweek).await? {
                bail!("Gameweek {} has already kicked off", gameweek);
            }

            let squad_info = fantasy_team_model::get_squad_player_info(pool, &entry.players).await?;
            if squad_info.len() != entry.players.len() {
                bail!("One or more selected players do not exist");
            }

            let starting_ids: std::collections::HashSet<i32> =
                entry.starting_ids.iter().copied().collect();

            squad_rules::validate_squad(
                &squad_info,
                &starting_ids,
                entry.captain_id,
                entry.vice_captain_id,
            )?;

            serde_json::to_value(&entry)?
        }

        other => bail!("Unknown contest type: {}", other),
    };

    let result = sqlx::query!(
        r#"
        INSERT INTO fantasy_league_participants
        (league_id, user_id, user_name, team_data)
        VALUES ($1, $2, $3, $4)
        "#,
        league_id,
        user_id,
        user_name,
        Json(team_data) as _,
    )
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
            bail!("You have already joined this league")
        }
        Err(e) => Err(e.into()),
    }
}
