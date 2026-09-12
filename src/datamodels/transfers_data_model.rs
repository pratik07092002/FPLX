use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct TransferItem {
    pub player_out_id: i32,
    pub player_in_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct MakeTransfersRequest {
    pub transfers: Vec<TransferItem>,
}

#[derive(Debug, Serialize)]
pub struct TransferSummary {
    pub gameweek: i32,
    pub transfers_made: usize,
    pub free_transfers_used: usize,
    pub hits: usize,
    pub points_cost: i32,
    pub free_transfers_remaining: i32,
}
