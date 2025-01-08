use axum::{routing::get, Router};
use sqlx::PgPool;

// -- 配置所有路由
pub fn create_router(pool: PgPool) -> Router {
    Router::new().route("/", get(health_check)).with_state(pool)
}

// -- 健康检查接口
async fn health_check() -> &'static str {
    "Hello, Axum!"
}
