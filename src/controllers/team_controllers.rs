use crate::helpers::response_helper;
use crate::models::team_model;
use crate::{datamodels::official_fpl_models::{
    FplBootstrapResponse,
    FplTeam
}, helpers::http_client::HttpClient};
use actix_web::{HttpResponse, web};
use anyhow::Result;
use sqlx::PgPool;

pub async fn sync_teams(pool: web::Data<PgPool>) -> HttpResponse {
    match sync(pool.get_ref()).await {

        Ok(teams) => {

            let res = response_helper::success("Teams fetched successfully",
             teams);

             HttpResponse::Ok().json(res)
        }

        Err(e) => {


            let res = response_helper::failure("Failed to fetch teams",
             500,  None::<Vec<FplTeam>>);

             HttpResponse::InternalServerError().json(res)
        } 
    }
}

async fn sync(pool: &PgPool) -> Result<Vec<FplTeam>>{
    let client = HttpClient::new();
    let res = client
        .get::<FplBootstrapResponse>(
            "https://fantasy.premierleague.com/api/bootstrap-static",
        )
        .await?;
    let teams = res.teams;
team_model::upsert_teams(pool, &teams ).await?;
    Ok(teams)
}
