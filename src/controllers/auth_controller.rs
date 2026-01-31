use actix_web::{HttpResponse, web};
use base64::Engine;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use jsonwebtoken::{EncodingKey, Header, encode};
use rand::{Rng, distributions::Alphanumeric};
use sqlx::PgPool;

use crate::{
    datamodels::auth_models::{
        AuthResponse, JwtClaims, NonceRequest, NonceResponse, VerifyRequest,
    },
    helpers::response_helper,
    models::auth_model,
};

pub async fn get_nonce(
    pool: web::Data<PgPool>,
    req: web::Json<NonceRequest>,
) -> HttpResponse {

    let nonce: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let res = auth_model::insert_nonce(
        pool.get_ref(),
        &nonce,
        req.wallet.clone(),
    )
    .await;

    if let Err(e) = res {
        eprintln!("Insert nonce error: {:?}", e);

        return HttpResponse::InternalServerError().json(
            response_helper::failure(
                "Failed to generate nonce",
                500,
            ),
        );
    }

    HttpResponse::Ok().json(
        response_helper::success(
            "Nonce generated",
            NonceResponse { nonce },
        ),
    )
}


pub async fn verify_wallet(pool: web::Data<PgPool>, req: web::Json<VerifyRequest>) -> HttpResponse {
    // GET NONCE

    let nonce = match auth_model::get_nonce(pool.get_ref(), &req.wallet).await {
        Ok(n) => n,

        Err(e) => {
            eprintln!("Nonce error: {:?}", e);

            return HttpResponse::Unauthorized()
                .json(response_helper::failure("Invalid wallet", 401));
        }
    };

    // DECODE PUBLIC KEY

    let pubkey_bytes = match bs58::decode(&req.wallet).into_vec() {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(response_helper::failure("Invalid address", 401));
        }
    };

    let pubkey_array: [u8; 32] = match pubkey_bytes.try_into() {
        Ok(a) => a,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(response_helper::failure("Invalid public key length", 401));
        }
    };

    let key = match VerifyingKey::from_bytes(&pubkey_array) {
        Ok(k) => k,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(response_helper::failure("Invalid public key", 401));
        }
    };

    // DECODE SIGNATURE

    let sig_bytes = match base64::engine::general_purpose::STANDARD.decode(&req.signature) {
        Ok(v) => v,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(response_helper::failure("Invalid signature encoding", 401));
        }
    };

    let sig_array: [u8; 64] = match sig_bytes.try_into() {
        Ok(a) => a,
        Err(_) => {
            return HttpResponse::Unauthorized()
                .json(response_helper::failure("Invalid signature length", 401));
        }
    };

    let sig = Signature::from_bytes(&sig_array);

    // VERIFY

    if key.verify(nonce.as_bytes(), &sig).is_err() {
        return HttpResponse::Unauthorized()
            .json(response_helper::failure("Verification failed", 401));
    }
auth_model::mark_logged_in(
    pool.get_ref(),
    &req.wallet,
).await;
    // CREATE JWT

    let claims = JwtClaims {
        sub: req.wallet.clone(),
        exp: (chrono::Utc::now().timestamp() + 86400) as usize, // 24h
    };

    let token = match encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(b"SUPER_SECRET_KEY"),
    ) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JWT error: {:?}", e);

            return HttpResponse::InternalServerError()
                .json(response_helper::failure("Token generation failed", 500));
        }
    };

    // SUCCESS

    HttpResponse::Ok().json(response_helper::success(
        "Wallet verified",
        AuthResponse { token },
    ))
}
