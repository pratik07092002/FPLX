use serde::{Deserialize, Serialize};

/// A player-picked squad, submitted directly on a Derby/Round participant
/// row (no persistent fantasy_teams entry — one-shot for that contest).
#[derive(Debug, Serialize, Deserialize)]
pub struct ContestEntry {
    pub players: Vec<i32>,
    pub starting_ids: Vec<i32>,
    pub captain_id: i32,
    pub vice_captain_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateLeagueRequest {
    pub league_name: String,
    pub league_type: String, // "public" | "private"

    #[serde(default = "default_contest_type")]
    pub contest_type: String, // "campaign" | "derby" | "round"

    pub fixture_id: Option<i32>,
    pub gameweek: Option<i32>,
}

fn default_contest_type() -> String {
    "campaign".to_string()
}

#[derive(Debug, Deserialize)]
pub struct JoinLeagueRequest {
    pub join_code: Option<String>,

    /// Required for derby/round leagues — ignored for campaign (which scores
    /// off the member's persistent fantasy_teams squad instead).
    pub entry: Option<ContestEntry>,
}

#[derive(Debug, Serialize)]
pub struct LeagueResponse {
    pub id: i32,
    pub league_name: String,
    pub league_type: String,
    pub contest_type: String,
    pub fixture_id: Option<i32>,
    pub gameweek: Option<i32>,
    pub join_code: Option<String>,
}
