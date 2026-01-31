use crate::datamodels::official_fpl_models::FPLPlayer;
use crate::helpers::response_helper;
use crate::models::official_fpl_sync_model;
use crate::{
    datamodels::official_fpl_models::{FplBootstrapResponse, FplBootstrapResponsePlayers, FplTeam},
    helpers::http_client::HttpClient,
};
use actix_web::{HttpResponse, web};
use anyhow::Result;
use sqlx::PgPool;

pub async fn sync_teams(pool: web::Data<PgPool>) -> HttpResponse {
    match sync_fpl_team(pool.get_ref()).await {
        Ok(teams) => {
            let res = response_helper::success("Teams fetched successfully", teams);

            HttpResponse::Ok().json(res)
        }

        Err(e) => {
            let res = response_helper::failure("Failed to fetch teams", 500);

            HttpResponse::InternalServerError().json(res)
        }
    }
}
pub async fn sync_players(pool: web::Data<PgPool>) -> HttpResponse {
    match sync_fpl_players(pool.get_ref()).await {
        Ok(players) => {
            let res = response_helper::success("Players Synced Successfully", players);
            HttpResponse::Ok().json(res)
        }

        Err(e) => {
            println!("Error occured : {:?}", e);
            let res = response_helper::failure("Player sync failed", 400);
            HttpResponse::InternalServerError().json(res)
        }
    }
}

async fn sync_fpl_team(pool: &PgPool) -> Result<Vec<FplTeam>> {
    let client = HttpClient::new();
    let res = client
        .get::<FplBootstrapResponse>("https://fantasy.premierleague.com/api/bootstrap-static")
        .await?;
    let teams = res.teams;
    official_fpl_sync_model::upsert_teams(pool, &teams).await?;
    Ok(teams)
}

async fn sync_fpl_players(pool: &PgPool) -> Result<Vec<FPLPlayer>> {
    let client = HttpClient::new();
    let res = client
        .get::<FplBootstrapResponsePlayers>(
            "https://fantasy.premierleague.com/api/bootstrap-static",
        )
        .await?;
    let players = res.elements;
    official_fpl_sync_model::upsert_players(pool, &players).await?;

    Ok(players)
}
