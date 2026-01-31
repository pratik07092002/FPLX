use sqlx::PgPool;
use std::env;

pub async fn connect_db() -> Result<PgPool, sqlx::Error>{
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");

    PgPool::connect(&db_url).await
}