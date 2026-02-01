use sqlx::PgPool;
use anyhow::Result;

use crate::datamodels::auth_models::UserAuth;

pub async fn insert_nonce(
    pool: &PgPool,
    nonce: &str,
    wallet: String,
) -> Result<()> {

    sqlx::query!(
        r#"
        INSERT INTO users (wallet_address, nonce)
        VALUES ($1,$2)
        ON CONFLICT (wallet_address)
        DO UPDATE SET nonce = $2
        "#,
        wallet,
        nonce
    )
    .execute(pool)
    .await?; 

    Ok(())
}

pub async fn get_nonce(
    pool: &PgPool,
    wallet: &str,
) -> Result<String> {




    let rec = sqlx::query!(
        "SELECT nonce FROM users WHERE wallet_address = $1",
        wallet
    )
    .fetch_one(pool)
    .await?;

    let nonce = rec.nonce
        .ok_or_else(|| anyhow::anyhow!("Nonce not found"))?;

    Ok(nonce)
}

pub async fn mark_logged_in(
    pool: &PgPool,
    wallet: &str,
) -> anyhow::Result<()> {

    sqlx::query!(
        r#"
        UPDATE users
        SET
            last_login = now(),
            nonce = NULL
        WHERE wallet_address = $1
        "#,
        wallet
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_user_by_wallet(
    pool: &PgPool,
    wallet: &str,
) -> Result<UserAuth, sqlx::Error> {

    let rec = sqlx::query!(
        r#"
        SELECT id
        FROM users
        WHERE wallet_address = $1
        "#,
        wallet
    )
    .fetch_one(pool)
    .await?;

    Ok(UserAuth {
        id: rec.id,
    })
}
