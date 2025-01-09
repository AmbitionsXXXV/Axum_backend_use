use axum::{
    extract::Request,
    http::{header, Method, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Extension,
};
use std::sync::Arc;
use std::time::Instant;

use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    db::UserExt,
    error::{ErrorMessage, HttpError},
    models::{User, UserRole},
    utils::token,
    AppState,
};

// -- 请求日志中间件
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    // -- 提取请求信息
    let method: Method = request.method().clone();
    let uri: Uri = request.uri().clone();
    let start: Instant = Instant::now();

    // -- 处理请求
    let response: Response = next.run(request).await;

    // -- 获取响应状态码
    let status: StatusCode = response.status();
    let duration = start.elapsed();

    // -- 记录请求信息
    match status.as_u16() {
        200..=299 => {
            tracing::info!(
                target: "request",
                method = %method,
                path = %uri,
                status = %status.as_u16(),
                duration = ?duration,
                "请求成功"
            );
        }
        400..=499 => {
            tracing::warn!(
                target: "request",
                method = %method,
                path = %uri,
                status = %status.as_u16(),
                duration = ?duration,
                "客户端错误"
            );
        }
        _ => {
            tracing::error!(
                target: "request",
                method = %method,
                path = %uri,
                status = %status.as_u16(),
                duration = ?duration,
                "服务器错误"
            );
        }
    }

    response
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddeware {
    pub user: User,
}

pub async fn auth(
    cookie_jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError> {
    let cookies = cookie_jar
        .get("token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let token = cookies
        .ok_or_else(|| HttpError::unauthorized(ErrorMessage::TokenNotProvided.to_string()))?;

    let token_details = match token::decode_token(token, app_state.env.jwt_secret.as_bytes()) {
        Ok(token_details) => token_details,
        Err(_) => {
            return Err(HttpError::unauthorized(
                ErrorMessage::InvalidToken.to_string(),
            ));
        }
    };

    let user_id = uuid::Uuid::parse_str(&token_details.to_string())
        .map_err(|_| HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()))?;

    let user = app_state
        .db_client
        .get_user(Some(user_id), None, None, None)
        .await
        .map_err(|_| HttpError::unauthorized(ErrorMessage::UserNoLongerExist.to_string()))?;

    let user =
        user.ok_or_else(|| HttpError::unauthorized(ErrorMessage::UserNoLongerExist.to_string()))?;

    req.extensions_mut()
        .insert(JWTAuthMiddeware { user: user.clone() });

    Ok(next.run(req).await)
}

pub async fn role_check(
    Extension(_app_state): Extension<Arc<AppState>>,
    req: Request,
    next: Next,
    required_roles: Vec<UserRole>,
) -> Result<impl IntoResponse, HttpError> {
    let user = req
        .extensions()
        .get::<JWTAuthMiddeware>()
        .ok_or_else(|| HttpError::unauthorized(ErrorMessage::UserNotAuthenticated.to_string()))?;

    if !required_roles.contains(&user.user.role) {
        return Err(HttpError::new(
            ErrorMessage::PermissionDenied.to_string(),
            StatusCode::FORBIDDEN,
        ));
    }

    Ok(next.run(req).await)
}
