use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Deserializer};
use uuid::Uuid;

fn deserialize_form<'de, D>(
    deserializer: D,
) -> Result<Option<f32>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<serde_json::Value> =
        Option::deserialize(deserializer)?;

    match value {
        None => Ok(None),

        Some(serde_json::Value::String(s)) => {
            s.parse::<f32>()
                .map(Some)
                .map_err(serde::de::Error::custom)
        }

        Some(serde_json::Value::Number(n)) => {
            Ok(n.as_f64().map(|v| v as f32))
        }

        _ => Err(serde::de::Error::custom("invalid form"))
    }
}

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
#[serde(deserialize_with = "deserialize_form")]
    pub form: Option<f32>,

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

// =====================================================
// LIVE SYNC (fixtures + in-play stats)
// =====================================================

#[derive(Debug, Deserialize)]
pub struct FplEvent {
    pub id: i32,
    pub is_current: bool,
}

#[derive(Debug, Deserialize)]
pub struct FplBootstrapEvents {
    pub events: Vec<FplEvent>,
}

#[derive(Debug, Deserialize)]
pub struct FplFixture {
    pub id: i32,
    pub event: Option<i32>,
    pub kickoff_time: Option<DateTime<Utc>>,
    pub started: Option<bool>,
    pub finished: Option<bool>,
    pub team_h: Option<i32>,
    pub team_a: Option<i32>,
    pub team_h_score: Option<i32>,
    pub team_a_score: Option<i32>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct FplLiveStats {
    #[serde(default)]
    pub minutes: i32,
    #[serde(default)]
    pub goals_scored: i32,
    #[serde(default)]
    pub assists: i32,
    #[serde(default)]
    pub own_goals: i32,
    #[serde(default)]
    pub penalties_saved: i32,
    #[serde(default)]
    pub penalties_missed: i32,
    #[serde(default)]
    pub yellow_cards: i32,
    #[serde(default)]
    pub red_cards: i32,
    #[serde(default)]
    pub saves: i32,
    #[serde(default)]
    pub bonus: i32,
    #[serde(default)]
    pub bps: i32,
    #[serde(default)]
    pub defensive_contribution: i32,
}

#[derive(Debug, Deserialize)]
pub struct FplExplain {
    pub fixture: i32,
}

#[derive(Debug, Deserialize)]
pub struct FplLiveElement {
    pub id: i32,
    pub stats: FplLiveStats,
    #[serde(default)]
    pub explain: Vec<FplExplain>,
}

#[derive(Debug, Deserialize)]
pub struct FplLiveResponse {
    pub elements: Vec<FplLiveElement>,
}


