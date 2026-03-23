/*
PostgreSQL 数据库迁移管理
*/

use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use std::env;

pub type DbPool = Pool<Postgres>;

/// 创建 PostgreSQL 数据库连接池
pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://mytodoapp:mytodoapp@localhost:5432/mytodoapp_db".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(20)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

/// PostgreSQL 数据库连接测试
pub async fn create_database_if_not_exists(database_url: &str) -> Result<(), sqlx::Error> {
    tracing::info!("检查 PostgreSQL 数据库连接: {}", database_url);

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await?;

    sqlx::query("SELECT 1").fetch_one(&pool).await?;
    tracing::info!("PostgreSQL 数据库连接正常");

    Ok(())
}

/// 运行 PostgreSQL 数据库迁移
pub async fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("运行 PostgreSQL 数据库迁移...");

    // 创建迁移表
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT NOW(),
            success BOOLEAN NOT NULL,
            checksum BYTEA NOT NULL,
            execution_time BIGINT NOT NULL
        )"#,
    )
    .execute(pool)
    .await?;

    // 运行自定义迁移
    run_custom_migrations(pool).await?;

    tracing::info!("PostgreSQL 数据库迁移完成");
    Ok(())
}

/// 运行自定义迁移脚本
async fn run_custom_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await?;
    tracing::info!("自定义迁移完成");

    Ok(())
}

/// 初始化数据库
pub async fn init_database() -> Result<DbPool, Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://mytodoapp:mytodoapp@localhost:5432/mytodoapp_db".to_string()
    });
    create_database_if_not_exists(&database_url).await?;
    let pool = create_pool().await?;
    run_migrations(&pool).await?;
    Ok(pool)
}
