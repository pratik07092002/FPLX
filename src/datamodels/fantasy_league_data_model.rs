use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FantasyLeague {
    pub id: i32,
    pub league_name: String,
    pub created_by_user_id: i32,
    pub created_by_user_name: String,
    pub league_type: String,
    pub join_code: Option<String>,
    pub max_participants: i32,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct LeagueParticipant {
    pub id: i32,
    pub league_id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub team_data: serde_json::Value,
    pub total_points: i32,
}



#[derive(Debug, Deserialize)]
pub struct CreateLeagueRequest {
    pub league_name: String,
    pub league_type: String, // "public" | "private"
}

#[derive(Debug, Serialize)]
pub struct LeagueResponse {
    pub id: i32,
    pub league_name: String,
    pub league_type: String,
    pub join_code: Option<String>,
}
