use actix_web::{web, App, HttpResponse, HttpServer};
use dotenvy::dotenv;
pub mod datamodels{
    pub mod official_fpl_models;
    pub mod api_response;
    pub mod auth_models;
    pub mod fantasy_team_data_models;
    pub mod fantasy_league_data_model;
    pub mod points_data_model;
    pub mod transfers_data_model;

}
pub mod helpers{
    pub mod db;
    pub mod response_helper;
    pub mod http_client;
    pub mod auth_middleware;
    pub mod fantasy_league_helper;
    pub mod fpl_meta;
    pub mod jwt;
    pub mod scoring_rules;
    pub mod squad_rules;
}
pub mod models {
    pub mod official_fpl_sync_model;
    pub mod auth_model;
    pub mod fantasy_team_model;
    pub mod fantasy_league_model;
    pub mod live_sync_model;
    pub mod points_engine_model;
    pub mod transfers_model;
}
pub mod routes {
    pub mod sync_routes;
}
pub mod controllers {
    pub mod offcial_fpl_controllers;
    pub mod auth_controller;
    pub mod fantasy_team_controllers;
    pub mod fantasy_league_controller;
    pub mod live_sync_controller;
    pub mod points_controller;
    pub mod transfers_controller;
}

const LIVE_POLL_SECS: u64 = 60;
const IDLE_POLL_SECS: u64 = 900;

async fn run_live_sync_loop(pool: sqlx::PgPool) {
    loop {
        let sleep_secs = match controllers::live_sync_controller::sync_live_gameweek(&pool).await {
            Ok(summary) => {
                println!(
                    "Live sync ok: GW{} ({} fixture(s) in play, {} players scored)",
                    summary.gameweek, summary.fixtures_live, summary.players_scored
                );
                if summary.fixtures_live > 0 {
                    LIVE_POLL_SECS
                } else {
                    IDLE_POLL_SECS
                }
            }
            Err(e) => {
                eprintln!("Live sync failed: {:?}", e);
                IDLE_POLL_SECS
            }
        };

        tokio::time::sleep(std::time::Duration::from_secs(sleep_secs)).await;
    }
}

async fn health(pool: web::Data<sqlx::PgPool>) -> HttpResponse {
    match sqlx::query!("SELECT 1 AS ok").fetch_one(pool.get_ref()).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "status": "ok" })),
        Err(e) => {
            eprintln!("Health check DB ping failed: {:?}", e);
            HttpResponse::ServiceUnavailable().json(serde_json::json!({ "status": "db_unreachable" }))
        }
    }
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {

    dotenv().ok();

    let pool = helpers::db::connect_db()
        .await?;

    println!("DB connected");

    // ignore_missing: a backfill migration was added with an earlier
    // timestamp than migrations already applied on existing databases.
    sqlx::migrate!("./migrations")
        .set_ignore_missing(true)
        .run(&pool)
        .await?;
    println!("Migrations up to date");

    tokio::spawn(run_live_sync_loop(pool.clone()));

    HttpServer::new(move || {

        App::new()
            .app_data(
                actix_web::web::Data::new(pool.clone())
            )
            .route("/health", web::get().to(health))
            .configure(
                routes::sync_routes::init
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
