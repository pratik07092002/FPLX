use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PlayerListItem {
    pub id: i32,
    pub first_name: String,
    pub second_name: String,
    pub photo: Option<String>,
    pub team_id: i32,
    pub team_name: String,
    pub team_short_name: String,
    pub position: i32,
    pub now_cost: i32,
    pub form: Option<f32>,
    pub total_points: Option<i32>,
    pub news: Option<String>,
}
