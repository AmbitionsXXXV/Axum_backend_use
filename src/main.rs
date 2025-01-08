mod config;
mod db;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;

use axum::middleware::from_fn;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    // -- 加载环境变量
    dotenv().ok();

    // -- 初始化日志
    tracing_subscriber::fmt::init();

    // -- 加载配置
    let config = config::Config::from_env();

    // -- 创建数据库连接池
    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    // -- 创建应用路由
    let app = routes::create_router(pool).layer(from_fn(middleware::logging_middleware));

    // -- 启动服务器
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server_port))
        .await
        .unwrap();

    tracing::info!("Server running on port {}", config.server_port);
    axum::serve(listener, app).await.unwrap();
}
