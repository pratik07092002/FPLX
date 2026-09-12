use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::datamodels::auth_models::AuthUser;
use crate::helpers::{fpl_meta, response_helper};
use crate::models::points_engine_model;

pub async fn my_points(pool: web::Data<PgPool>, user: AuthUser) -> HttpResponse {
    let gameweek = match fpl_meta::current_gameweek().await {
        Ok(gw) => gw,
        Err(e) => return HttpResponse::BadRequest().json(response_helper::failure(&e.to_string(), 400)),
    };

    match points_engine_model::get_team_points(pool.get_ref(), user.user_id, gameweek).await {
        Ok(points) => HttpResponse::Ok().json(response_helper::success("Points fetched", points)),
        Err(e) => {
            eprintln!("Get my points error: {:?}", e);
            HttpResponse::NotFound().json(response_helper::failure("No team found for this user", 404))
        }
    }
}

pub async fn league_leaderboard(
    pool: web::Data<PgPool>,
    path: web::Path<i32>,
) -> HttpResponse {
    let league_id = path.into_inner();

    match points_engine_model::get_league_leaderboard(pool.get_ref(), league_id).await {
        Ok(entries) => HttpResponse::Ok().json(response_helper::success("Leaderboard fetched", entries)),
        Err(e) => {
            eprintln!("Get leaderboard error: {:?}", e);
            HttpResponse::InternalServerError().json(response_helper::failure("Failed to fetch leaderboard", 500))
        }
    }
}
