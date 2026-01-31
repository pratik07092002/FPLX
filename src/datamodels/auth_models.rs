use serde::{Deserialize, Serialize};

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


#[derive(Serialize)]
pub struct JwtClaims {
    pub sub: String,
    pub exp: usize,
}
