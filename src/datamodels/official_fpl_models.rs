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