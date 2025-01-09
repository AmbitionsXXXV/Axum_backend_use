#![allow(unused)]

mod config;
mod db;
mod dtos;
mod error;
mod handlers;
mod mail;
mod middleware;
mod models;
mod routes;
mod utils;

use std::sync::Arc;

use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    middleware::from_fn,
};
use config::Config;
use db::DBClient;
use dotenvy::dotenv;
use routes::create_router;
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

#[tokio::main]
async fn main() {
    // -- 加载环境变量
    dotenv().ok();

    // -- 初始化日志
    tracing_subscriber::fmt::init();

    // -- 加载配置
    let config = config::Config::from_env();

    // -- 创建数据库连接池
    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    };

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

    let db_client = DBClient::new(pool);
    let app_state = AppState {
        env: config.clone(),
        db_client,
    };

    let app = create_router(Arc::new(app_state.clone())).layer(cors.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &config.server_port))
        .await
        .unwrap();

    tracing::info!("Server running on port {}", config.server_port);
    axum::serve(listener, app).await.unwrap();
}
