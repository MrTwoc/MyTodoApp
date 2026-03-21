/*
数据库连接池
*/
// src/db/pool.rs
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::env;

pub type DbPool = Pool<Sqlite>;

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = SqlitePoolOptions::new()
        .max_connections(10) // 最大连接数
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

// 测试连接
pub async fn test_connection(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").fetch_one(pool).await?;
    Ok(())
}
