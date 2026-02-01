use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize,Serialize)]
pub struct FplTeam {
    pub id : i32 ,
    pub name: String , 
    pub short_name: String,
    pub position: i32
}


#[derive(Debug, Deserialize)]
pub struct FplBootstrapResponse {
    pub teams: Vec<FplTeam>,
}



#[derive(Debug, Deserialize, Serialize)]
pub struct FPLPlayer {
    pub id: i32,

    pub first_name: String,
    pub second_name: String,
    pub photo: String,

    pub team: i32,

    // API sends this as "0.0" (string)
    pub form: String,
#[serde(default)]
    pub points: i32,
    pub total_points: i32,
    pub minutes: i32,

    pub goals_scored: i32,
    pub assists: i32,
    pub yellow_cards: i32,
    pub red_cards: i32,
    pub saves: i32,
    pub starts: i32,

    // Can be null
    pub news: Option<String>,
   #[serde(rename = "element_type")]
    pub position: i32
}

#[derive(Debug, Deserialize)]
pub struct FplBootstrapResponsePlayers {
    pub elements: Vec<FPLPlayer>,
}


#[derive(Debug, Serialize)]
pub struct MyTeamResponse {
    pub team_id: Uuid,
    pub captain: FPLPlayer,
    pub vice_captain: FPLPlayer,
    pub players: Vec<FPLPlayer>,
}
