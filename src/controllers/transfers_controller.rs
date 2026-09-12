use actix_web::{HttpResponse, web};
use sqlx::PgPool;

use crate::datamodels::auth_models::AuthUser;
use crate::datamodels::transfers_data_model::MakeTransfersRequest;
use crate::helpers::{fpl_meta, response_helper};
use crate::models::transfers_model;

pub async fn make_transfers(
    pool: web::Data<PgPool>,
    user: AuthUser,
    body: web::Json<MakeTransfersRequest>,
) -> HttpResponse {
    let gameweek = match fpl_meta::current_gameweek().await {
        Ok(gw) => gw,
        Err(e) => return HttpResponse::BadRequest().json(response_helper::failure(&e.to_string(), 400)),
    };

    match transfers_model::make_transfers(pool.get_ref(), user.user_id, gameweek, &body.transfers).await {
        Ok(summary) => HttpResponse::Ok().json(response_helper::success("Transfers applied", summary)),
        Err(e) => {
            eprintln!("Transfer error: {:?}", e);
            HttpResponse::BadRequest().json(response_helper::failure(&e.to_string(), 400))
        }
    }
}
