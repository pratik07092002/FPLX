use crate::controllers::offcial_fpl_controllers;
use actix_web::{post, web};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/sync").route(
        "/teams",
        web::post().to(offcial_fpl_controllers::sync_teams),
    ).route("/players", web::post().to(offcial_fpl_controllers::sync_players)));
}
