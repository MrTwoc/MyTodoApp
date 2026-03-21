/*
数据库迁移文件
*/

/*
数据库迁移管理
*/

use sqlx::{Pool, Sqlite, migrate::MigrateDatabase, sqlite::SqlitePoolOptions};
use std::env;

pub type DbPool = Pool<Sqlite>;

/// 创建数据库连接池
pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = SqlitePoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

/// 如果数据库不存在则创建
pub async fn create_database_if_not_exists(database_url: &str) -> Result<(), sqlx::Error> {
    if !Sqlite::database_exists(database_url).await? {
        tracing::info!("Creating database: {}", database_url);
        Sqlite::create_database(database_url).await?;
    }
    Ok(())
}

/// 运行所有迁移
pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    tracing::info!("Running database migrations...");

    sqlx::migrate!("./migrations").run(pool).await?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

/// 初始化数据库（创建数据库 + 运行迁移）
pub async fn init_database() -> Result<DbPool, Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")?;

    // 创建数据库（如果不存在）
    create_database_if_not_exists(&database_url).await?;

    // 创建连接池
    let pool = create_pool().await?;

    // 运行迁移
    run_migrations(&pool).await?;

    Ok(pool)
}

/// 回滚到指定版本（需要手动实现）
pub async fn rollback_to_version(pool: &DbPool, version: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM _sqlx_migrations WHERE version > $1")
        .bind(version)
        .execute(pool)
        .await?;

    tracing::warn!("Rolled back to version {}", version);
    Ok(())
}
