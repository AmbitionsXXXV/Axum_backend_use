use std::env;

// -- 应用配置结构体
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
}

impl Config {
    // -- 从环境变量加载配置
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("SERVER_PORT must be a valid number");

        Self {
            database_url,
            server_port,
        }
    }
}
