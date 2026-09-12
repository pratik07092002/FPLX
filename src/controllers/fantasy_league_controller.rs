use actix_web::{HttpResponse, web};

use crate::datamodels::auth_models::AuthUser;
use crate::datamodels::fantasy_league_data_model::{CreateLeagueRequest, JoinLeagueRequest};
use crate::helpers::fantasy_league_helper::generate_code;
use crate::helpers::response_helper;
use crate::models::fantasy_league_model;

use sqlx::PgPool;

pub async fn create_league(
    user: AuthUser,
    pool: web::Data<PgPool>,
    body: web::Json<CreateLeagueRequest>,
) -> HttpResponse {
    match handle_create(user, pool.get_ref(), body.into_inner()).await {
        Ok(league) => {
            let res = response_helper::success("League created successfully", league);

            HttpResponse::Ok().json(res)
        }

        Err(e) => {
            eprintln!("Create league error: {:?}", e);

            let err_str = e.to_string();

            let msg = if err_str.contains("Public league limit") {
                "You can create only 3 public leagues"
            } else if err_str.contains("Private league limit") {
                "You can create only 2 private leagues"
            } else if let Some(sqlx::Error::Database(db_err)) = e.downcast_ref::<sqlx::Error>() {
                match db_err.code().as_deref() {
                    // PostgreSQL unique violation
                    Some("23505") => "You already have a league with this name",

                    _ => "Database error",
                }
            } else {
                &err_str
            };

            let res = response_helper::failure(msg, 400);

            HttpResponse::BadRequest().json(res)
        }
    }
}

async fn handle_create(
    user: AuthUser,
    pool: &PgPool,
    body: CreateLeagueRequest,
) -> anyhow::Result<crate::datamodels::fantasy_league_data_model::LeagueResponse> {

    // Validate type
    if body.league_type != "public" && body.league_type != "private" {
        anyhow::bail!("Invalid league type");
    }

    let (fixture_id, gameweek) = match body.contest_type.as_str() {
        "campaign" => (None, None),

        "derby" => {
            let fixture_id = body
                .fixture_id
                .ok_or_else(|| anyhow::anyhow!("A derby league needs a fixture_id"))?;
            (Some(fixture_id), None)
        }

        "round" => {
            let gameweek = body
                .gameweek
                .ok_or_else(|| anyhow::anyhow!("A round league needs a gameweek"))?;
            (None, Some(gameweek))
        }

        other => anyhow::bail!("Unknown contest type: {}", other),
    };

    // Generate join code if private
    let join_code = if body.league_type == "private" {
        Some(generate_code())
    } else {
        None
    };

    // Start transaction
    let mut tx = pool.begin().await?;

    // Create league
    let league = fantasy_league_model::create_league_tx(
        &mut tx,
        &body.league_name,
        &body.league_type,
        &body.contest_type,
        fixture_id,
        gameweek,
        user.user_id,
        &user.wallet,
        join_code,
    )
    .await?;

    // Campaign leagues auto-join the creator on their existing squad.
    // Derby/Round need a squad submitted, so the creator joins separately
    // via /fantasy/leagues/{id}/join like everyone else.
    if body.contest_type == "campaign" {
        fantasy_league_model::add_participant_tx(
            &mut tx,
            league.id,
            user.user_id,
            &user.wallet,
        )
        .await?;
    }

    // Commit
    tx.commit().await?;

    Ok(league)
}

pub async fn join_league(
    user: AuthUser,
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
    body: web::Json<JoinLeagueRequest>,
) -> HttpResponse {
    let league_id = path.into_inner();
    let body = body.into_inner();

    match fantasy_league_model::join_league(
        pool.get_ref(),
        league_id,
        user.user_id,
        &user.wallet,
        body.join_code.as_deref(),
        body.entry,
    )
    .await
    {
        Ok(_) => HttpResponse::Ok().json(response_helper::success("Joined league", ())),
        Err(e) => {
            eprintln!("Join league error: {:?}", e);
            HttpResponse::BadRequest().json(response_helper::failure(&e.to_string(), 400))
        }
    }
}
