use actix_web::{HttpResponse, web};
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    datamodels::{auth_models::AuthUser, fantasy_team_data_models::CreateTeamRequest},
    helpers::response_helper,
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

    if req.players.len() != 15 {
        anyhow::bail!("Exactly 15 players required");
    }

    if req.captain_id == req.vice_captain_id {
        anyhow::bail!("Captain and VC same");
    }

    if !req.players.contains(&req.captain_id) {
        anyhow::bail!("Captain not in team");
    }

    if !req.players.contains(&req.vice_captain_id) {
        anyhow::bail!("VC not in team");
    }

    // ---------- TRANSACTION ----------

    let mut tx = pool.begin().await?;

    // Create team
    let team_id =
        fantasy_team_model::create_team(
            &mut tx,
            user_id,
            req.captain_id,
            req.vice_captain_id,
        )
        .await?;

    // Insert players
    fantasy_team_model::insert_players(
        &mut tx,
        team_id,
        &req.players,
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


