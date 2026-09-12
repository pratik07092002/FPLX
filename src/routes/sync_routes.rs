use crate::controllers::{auth_controller, catalog_controller, fantasy_league_controller, fantasy_team_controllers, live_sync_controller, offcial_fpl_controllers, points_controller, transfers_controller};
use actix_web::{post, web};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/teams", web::get().to(catalog_controller::list_teams));
    cfg.route("/players", web::get().to(catalog_controller::list_players));

    cfg.service(
        web::scope("/sync")
            .route(
                "/teams",
                web::post().to(offcial_fpl_controllers::sync_teams),
            )
            .route(
                "/players",
                web::post().to(offcial_fpl_controllers::sync_players),
            )
            .route(
                "/live",
                web::post().to(live_sync_controller::sync_live),
            ),
    );

    cfg.service(
        web::scope("/auth")
            .route("/nonce", web::post().to(auth_controller::get_nonce))
            .route("/verify", web::post().to(auth_controller::verify_wallet)),
    );

    cfg.service(
        web::scope("/fantasy")
            .route(
                "/create-team",
                web::post().to(fantasy_team_controllers::create_team),
            )
            .route(
                "/my-team",
                web::get().to(fantasy_team_controllers::get_my_team),
            )
             .route(
                "/create-league",
                web::post().to(fantasy_league_controller::create_league),
            )
            .route(
                "/leagues/{league_id}/join",
                web::post().to(fantasy_league_controller::join_league),
            )
            .route(
                "/my-points",
                web::get().to(points_controller::my_points),
            )
            .route(
                "/leagues/{league_id}/leaderboard",
                web::get().to(points_controller::league_leaderboard),
            )
            .route(
                "/transfers",
                web::post().to(transfers_controller::make_transfers),
            )
    );
}
