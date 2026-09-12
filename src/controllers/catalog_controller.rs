use actix_web::{web, HttpResponse};
use sqlx::PgPool;

use crate::helpers::response_helper;
use crate::models::catalog_model;

pub async fn list_teams(pool: web::Data<PgPool>) -> HttpResponse {
    match catalog_model::list_teams(pool.get_ref()).await {
        Ok(teams) => HttpResponse::Ok().json(response_helper::success("Teams fetched", teams)),
        Err(e) => {
            eprintln!("List teams error: {:?}", e);
            HttpResponse::InternalServerError().json(response_helper::failure("Failed to fetch teams", 500))
        }
    }
}

pub async fn list_players(pool: web::Data<PgPool>) -> HttpResponse {
    match catalog_model::list_players(pool.get_ref()).await {
        Ok(players) => HttpResponse::Ok().json(response_helper::success("Players fetched", players)),
        Err(e) => {
            eprintln!("List players error: {:?}", e);
            HttpResponse::InternalServerError().json(response_helper::failure("Failed to fetch players", 500))
        }
    }
}
