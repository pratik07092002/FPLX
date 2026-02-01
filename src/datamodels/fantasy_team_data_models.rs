use serde::{Deserialize,Serialize};
use uuid::Uuid;
use crate::datamodels::official_fpl_models::FPLPlayer;

#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {

    pub players: Vec<i32>,

    pub captain_id: i32,

    pub vice_captain_id: i32,
}


#[derive(Debug, Serialize)]
pub struct MyTeamResponse {
    pub team_id: Uuid,
    pub captain: FPLPlayer,
    pub vice_captain: FPLPlayer,
    pub players: Vec<FPLPlayer>,
}


