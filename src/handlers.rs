use crate::error::HttpError;
use crate::models::User;
use sqlx::PgPool;

// -- 用户相关的处理函数
pub async fn get_user(pool: &PgPool, user_id: uuid::Uuid) -> Result<User, HttpError> {
    // -- 示例处理函数，后续实现具体逻辑
    todo!("Implement get_user handler")
}
