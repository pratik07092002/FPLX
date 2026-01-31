use actix_web::{App, HttpServer};
use dotenvy::dotenv;
pub mod datamodels{
    pub mod official_fpl_models;
    pub mod api_response;
}
pub mod helpers{
    pub mod db;
    pub mod response_helper;
    pub mod http_client;
}
pub mod models {
    pub mod team_model;
}
pub mod routes {
    pub mod sync_routes;
}
pub mod controllers {
    pub mod team_controllers;
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {

    dotenv().ok();

    let pool = helpers::db::connect_db()
        .await?; 

    println!("DB connected");

    HttpServer::new(move || {

        App::new()
            .app_data(
                actix_web::web::Data::new(pool.clone())
            )
            .configure(
                routes::sync_routes::init
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}
