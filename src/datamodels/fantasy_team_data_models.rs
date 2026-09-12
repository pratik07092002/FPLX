use serde::{Deserialize,Serialize};
use uuid::Uuid;
use crate::datamodels::official_fpl_models::FPLPlayer;

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {

    /// Exactly 15 player IDs — the full squad.
    pub players: Vec<i32>,

    /// Exactly 11 of the IDs above — this gameweek's starting lineup.
    pub starting_ids: Vec<i32>,

    pub captain_id: i32,

    pub vice_captain_id: i32,
}


#[derive(Debug, Serialize)]
pub struct SquadPlayerView {
    #[serde(flatten)]
    pub player: FPLPlayer,
    pub is_starting: bool,
    pub bench_order: Option<i16>,
}


#[derive(Debug, Serialize)]
pub struct MyTeamResponse {
    pub team_id: Uuid,
    pub captain: FPLPlayer,
    pub vice_captain: FPLPlayer,
    pub players: Vec<SquadPlayerView>,
}


