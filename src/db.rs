use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

// -- 创建数据库连接池
pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    // -- 从环境变量获取数据库URL
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // -- 配置连接池选项
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    // -- 运行简单查询来测试连接
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await?;

    Ok(pool)
}
