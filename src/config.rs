use std::env;

// -- 应用配置结构体
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_port: u16,
    pub jwt_secret: String,
    pub jwt_maxage: i64,
}

impl Config {
    /// 从环境变量加载配置
    ///
    /// 读取环境变量 `DATABASE_URL`, `JWT_SECRET_KEY`, `JWT_MAXAGE` 和 `SERVER_PORT`，并
    /// 将其加载到 `Config` 实例中。如果环境变量不存在或解析失败，将会 panic。
    ///
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let jwt_secret = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE")
            .expect("JWT_MAXAGE must be set")
            .parse()
            .expect("JWT_MAXAGE must be a number");

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("SERVER_PORT must be a valid number");

        Self {
            jwt_secret,
            jwt_maxage,
            database_url,
            server_port,
        }
    }
}
