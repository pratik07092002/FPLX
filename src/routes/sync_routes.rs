use crate::controllers::team_controllers;
use actix_web::{post, web};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/sync").route("/teams", web::post().to(team_controllers::sync_teams)));
}
