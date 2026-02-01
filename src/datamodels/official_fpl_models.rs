use serde::{Deserialize, Serialize};
use uuid::Uuid;

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



#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FPLPlayer {
    pub id: i32,

    pub first_name: Option<String>,
    pub second_name: Option<String>,
    pub photo: Option<String>,

    pub team: Option<i32>,

    pub form: Option<String>,

    #[serde(default)]
    pub points: Option<i32>,

    pub total_points: Option<i32>,
    pub minutes: Option<i32>,

    pub goals_scored: Option<i32>,
    pub assists: Option<i32>,
    pub yellow_cards: Option<i32>,
    pub red_cards: Option<i32>,
    pub saves: Option<i32>,
    pub starts: Option<i32>,

    pub news: Option<String>,

    #[serde(rename = "element_type")]
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct FplBootstrapResponsePlayers {
    pub elements: Vec<FPLPlayer>,
}


