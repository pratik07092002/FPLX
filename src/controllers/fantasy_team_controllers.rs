use actix_web::{HttpResponse, web};
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashSet;
use uuid::Uuid;

use crate::{
    datamodels::{auth_models::AuthUser, fantasy_team_data_models::CreateTeamRequest},
    helpers::{fpl_meta, response_helper, squad_rules},
    models::fantasy_team_model,
};

pub async fn create_team(
    pool: web::Data<PgPool>,

    user: AuthUser, 

    req: web::Json<CreateTeamRequest>,
) -> HttpResponse {
    let user_id = user.user_id;


    match process(pool.get_ref(), user_id, req.0).await {

        Ok(_) => {

            let res = response_helper::success(
                "Fantasy team created",
                (),
            );

            HttpResponse::Ok().json(res)
        }

        Err(e) => {

            eprintln!("Team create error: {:?}", e);

            let res = response_helper::failure(
                &e.to_string(),
                400,
            );

            HttpResponse::InternalServerError().json(res)
        }
    }
}


async fn process(
    pool: &PgPool,

    user_id: Uuid,

    req: CreateTeamRequest,
) -> Result<()> {

    // ---------- VALIDATION ----------

    let squad_info = fantasy_team_model::get_squad_player_info(pool, &req.players).await?;

    if squad_info.len() != req.players.len() {
        anyhow::bail!("One or more selected players do not exist");
    }

    let starting_ids: HashSet<i32> = req.starting_ids.iter().copied().collect();

    squad_rules::validate_squad(
        &squad_info,
        &starting_ids,
        req.captain_id,
        req.vice_captain_id,
    )?;

    let current_gameweek = fpl_meta::current_gameweek().await?;

    // ---------- TRANSACTION ----------

    let mut tx = pool.begin().await?;

    // Create team
    let team_id =
        fantasy_team_model::create_team(
            &mut tx,
            user_id,
            req.captain_id,
            req.vice_captain_id,
            current_gameweek,
        )
        .await?;

    // Insert players
    fantasy_team_model::insert_players(
        &mut tx,
        team_id,
        &req.players,
        &starting_ids,
    )
    .await?;

    // Update user
    fantasy_team_model::mark_team_created(
        &mut tx,
        user_id,
    )
    .await?;

    tx.commit().await?;

    Ok(())
}


pub async fn get_my_team(
    pool: web::Data<PgPool>,
    user: AuthUser,
) -> HttpResponse {

    match fantasy_team_model::get_my_team(
        pool.get_ref(),
        user.user_id,
    )
    .await
    {
        Ok(team) => {

            let res = response_helper::success(
                "Team fetched",
                team,
            );

            HttpResponse::Ok().json(res)
        }

        Err(e) => {

            eprintln!("Get team error: {:?}", e);

            let res = response_helper::failure(
                &e.to_string(),
                404,
            );

            HttpResponse::NotFound().json(res)
        }
    }
}


