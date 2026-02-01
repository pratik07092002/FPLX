use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {

    pub players: Vec<i32>,

    pub captain_id: i32,

    pub vice_captain_id: i32,
}


