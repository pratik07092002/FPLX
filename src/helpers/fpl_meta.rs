use anyhow::Result;

use crate::datamodels::official_fpl_models::FplBootstrapEvents;
use crate::helpers::http_client::HttpClient;

pub async fn current_gameweek() -> Result<i32> {
    let client = HttpClient::new();

    let bootstrap = client
        .get::<FplBootstrapEvents>("https://fantasy.premierleague.com/api/bootstrap-static/")
        .await?;

    bootstrap
        .events
        .iter()
        .find(|e| e.is_current)
        .map(|e| e.id)
        .ok_or_else(|| anyhow::anyhow!("No current gameweek (off-season or between seasons)"))
}
