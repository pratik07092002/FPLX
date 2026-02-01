use actix_web::{
    dev::Payload,
    web,
    Error,
    FromRequest,
    HttpRequest,
    HttpResponse,
};
use std::env;


use actix_web::error::InternalError;

use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::PgPool;

use std::future::Future;
use std::pin::Pin;

use crate::datamodels::auth_models::{AuthUser, JwtClaims};
use crate::models::auth_model;
use crate::helpers::response_helper;


fn unauthorized(msg: String) -> Error {

    let res = response_helper::failure(&msg, 401);

    InternalError::from_response(
        msg,
        HttpResponse::Unauthorized().json(res),
    )
    .into()
}


impl FromRequest for AuthUser {

    type Error = Error;

    type Future = Pin<
        Box<
            dyn Future<Output = Result<Self, Error>> + 'static
        >
    >;

    fn from_request(
        req: &HttpRequest,
        _: &mut Payload,
    ) -> Self::Future {

        let pool = req
            .app_data::<web::Data<PgPool>>()
            .cloned();

        let auth = req
            .headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        Box::pin(async move {

            // ---------- DB ----------

            let pool = match pool {
                Some(p) => p,
                None => {
                    return Err(unauthorized("No DB pool".to_string()));
                }
            };

            // ---------- DEV MODE ----------

let dev_mode = env::var("DEV_MODE")
    .unwrap_or("false".to_string());

if dev_mode == "true" {

let wallet = env::var("DEV_USER_WALLET")
    .unwrap_or("dev_wallet".to_string());


    let user =
        auth_model::get_user_by_wallet(&pool, &wallet)
            .await
            .map_err(|_| unauthorized("Dev user not found".to_string()))?;

    return Ok(AuthUser {
        user_id: user.id,
        wallet,
    });
}


            // ---------- Token ----------

            let auth = match auth {
                Some(a) => a,
                None => {
                    return Err(unauthorized("No token".to_string()));
                }
            };

            // ---------- Bearer ----------

            let token = match auth.strip_prefix("Bearer ") {
                Some(t) => t,
                None => {
                    return Err(unauthorized("Invalid token format".to_string()));
                }
            };

            // ---------- JWT ----------

            let decoded = match decode::<JwtClaims>(
                token,
                &DecodingKey::from_secret(b"SUPER_SECRET_KEY"),
                &Validation::default(),
            ) {
                Ok(d) => d,

                Err(e) => {

                    use jsonwebtoken::errors::ErrorKind;

                    let msg = match e.kind() {
                        ErrorKind::ExpiredSignature => "Token expired",
                        ErrorKind::InvalidToken => "Invalid token",
                        ErrorKind::InvalidSignature => "Invalid signature",
                        _ => "Unauthorized",
                    }
                    .to_string();

                    return Err(unauthorized(msg));
                }
            };

            let wallet = decoded.claims.sub;

            // ---------- DB Lookup ----------

            let user =
                auth_model::get_user_by_wallet(
                    &pool,
                    &wallet,
                )
                .await
                .map_err(|_| unauthorized("User not found".to_string()))?;

            // ---------- OK ----------

            Ok(AuthUser {
                user_id: user.id,
                wallet,
            })
        })
    }
}
