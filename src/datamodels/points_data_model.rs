use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct PlayerPoints {
    pub player_id: i32,
    pub first_name: String,
    pub second_name: String,
    pub position: i32,
    pub points: i32,
    pub is_captain: bool,
    pub is_vice_captain: bool,
}

#[derive(Debug, Serialize)]
pub struct TeamPoints {
    pub team_id: Uuid,
    pub gameweek: i32,
    pub total_points: i32,
    pub players: Vec<PlayerPoints>,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub user_id: Uuid,
    pub user_name: String,
    pub total_points: i32,
}
