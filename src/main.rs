mod db;

use axum::{
    routing::get,
    Router,
};

#[tokio::main]
async fn main() {
    // -- 加载环境变量
    dotenvy::dotenv().ok();
    
    // -- 初始化日志
    tracing_subscriber::fmt::init();
    
    // -- 创建数据库连接池
    let pool = db::create_pool()
        .await
        .expect("Failed to create database pool");
    
    // -- 创建应用路由
    let app = Router::new()
        .route("/", get(|| async { "Hello, Axum!" }))
        .with_state(pool);
    
    // -- 启动服务器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    println!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app)
        .await
        .unwrap();
}
