use actix_web::{HttpResponse, web};
use anyhow::Result;
use sqlx::PgPool;
use std::collections::HashMap;

use crate::datamodels::official_fpl_models::{FplFixture, FplLiveResponse};
use crate::helpers::{fpl_meta, http_client::HttpClient, response_helper};
use crate::models::{live_sync_model, points_engine_model};

pub struct LiveSyncSummary {
    pub gameweek: i32,
    pub fixtures_live: usize,
    pub players_scored: usize,
}

pub async fn sync_live(pool: web::Data<PgPool>) -> HttpResponse {
    match sync_live_gameweek(pool.get_ref()).await {
        Ok(summary) => {
            let res = response_helper::success(
                "Live sync complete",
                serde_json::json!({
                    "gameweek": summary.gameweek,
                    "fixtures_live": summary.fixtures_live,
                    "players_scored": summary.players_scored,
                }),
            );
            HttpResponse::Ok().json(res)
        }
        Err(e) => {
            eprintln!("Live sync error: {:?}", e);
            let res = response_helper::failure(&e.to_string(), 500);
            HttpResponse::InternalServerError().json(res)
        }
    }
}

pub async fn sync_live_gameweek(pool: &PgPool) -> Result<LiveSyncSummary> {
    let client = HttpClient::new();

    let gameweek = fpl_meta::current_gameweek().await?;

    let fixtures = client
        .get::<Vec<FplFixture>>(&format!(
            "https://fantasy.premierleague.com/api/fixtures/?event={}",
            gameweek
        ))
        .await?;

    live_sync_model::upsert_fixtures(pool, gameweek, &fixtures).await?;

    let fixtures_live = fixtures
        .iter()
        .filter(|f| f.started.unwrap_or(false) && !f.finished.unwrap_or(false))
        .count();

    let fixture_sides: HashMap<i32, (i32, i32)> = fixtures
        .iter()
        .filter_map(|f| match (f.team_h, f.team_a) {
            (Some(h), Some(a)) => Some((f.id, (h, a))),
            _ => None,
        })
        .collect();

    let live = client
        .get::<FplLiveResponse>(&format!(
            "https://fantasy.premierleague.com/api/event/{}/live/",
            gameweek
        ))
        .await?;

    let player_team = live_sync_model::get_player_team_map(pool).await?;

    live_sync_model::upsert_live_stats(pool, &live.elements, &player_team, &fixture_sides).await?;

    let players_scored = points_engine_model::recalculate_gameweek(pool, gameweek).await?;

    Ok(LiveSyncSummary {
        gameweek,
        fixtures_live,
        players_scored,
    })
}
