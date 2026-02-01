use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct NonceRequest {
    pub wallet: String,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub wallet: String,
    pub signature: String,
}

#[derive(Deserialize, Serialize)]
pub struct NonceResponse {
    pub nonce: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}


#[derive(Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub wallet: String,
}


pub struct UserAuth {
    pub id: Uuid,
}