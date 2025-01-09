use std::sync::Arc;

use axum::{
    extract::Query,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Extension, Json, Router,
};
use axum_extra::extract::cookie::Cookie;
use chrono::{Duration, Utc};
use validator::Validate;

use crate::{
    db::UserExt,
    dtos::{
        ForgotPasswordRequestDto, LoginUserDto, RegisterUserDto, ResetPasswordRequestDto, Response,
        UserLoginResponseDto, VerifyEmailQueryDto,
    },
    error::{ErrorMessage, HttpError},
    mail::mails::{send_forgot_password_email, send_verification_email, send_welcome_email},
    utils::{password, token},
    AppState,
};

pub fn auth_handler() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/verify", get(verify_email))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
}

/// 处理用户注册请求 -- 创建新用户并发送验证邮件
///
/// # 参数
/// - `app_state` -- 应用程序状态，包含数据库连接等共享资源
/// - `body` -- 用户注册请求体，包含用户名、邮箱和密码
///
/// # 返回
/// - `Ok(Response)` -- 注册成功，返回成功消息
/// - `Err(HttpError)` -- 注册失败，返回错误信息
///   - `BadRequest` -- 请求参数验证失败
///   - `UniqueViolation` -- 邮箱已存在
///   - `ServerError` -- 服务器内部错误
pub async fn register(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<RegisterUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    // -- 验证请求参数
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    // -- 生成验证令牌和过期时间
    let verification_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::hours(24);

    // -- 对密码进行哈希处理
    let hash_password =
        password::hash(&body.password).map_err(|e| HttpError::server_error(e.to_string()))?;

    // -- 保存用户信息到数据库
    let result = app_state
        .db_client
        .save_user(
            &body.name,
            &body.email,
            &hash_password,
            &verification_token,
            expires_at,
        )
        .await;

    match result {
        Ok(_user) => {
            // -- 异步发送验证邮件
            let email = body.email.clone();
            let name = body.name.clone();
            let token = verification_token.clone();
            tokio::spawn(async move {
                if let Err(e) = send_verification_email(&email, &name, &token).await {
                    eprintln!("Failed to send verification email: {}", e);
                }
            });

            // -- 返回注册成功响应
            Ok((
                StatusCode::CREATED,
                Json(Response {
                    status: "success",
                    message:
                        "Registration successful! Please check your email to verify your account."
                            .to_string(),
                }),
            ))
        }
        // -- 处理数据库错误
        Err(sqlx::Error::Database(db_err)) => {
            // -- 处理唯一约束违反（邮箱已存在）
            if db_err.is_unique_violation() {
                Err(HttpError::unique_constraint_violation(
                    ErrorMessage::EmailExist.to_string(),
                ))
            } else {
                // -- 处理其他数据库错误
                Err(HttpError::server_error(db_err.to_string()))
            }
        }
        // -- 处理其他错误
        Err(e) => Err(HttpError::server_error(e.to_string())),
    }
}

/// 处理用户登录请求 -- 验证用户身份并生成访问令牌
///
/// # 参数
/// - `app_state` -- 应用程序状态，包含数据库连接等共享资源
/// - `body` -- 登录请求体，包含邮箱和密码
///
/// # 返回
/// - `Ok(Response)` -- 登录成功，返回访问令牌和用户信息
/// - `Err(HttpError)` -- 登录失败，返回错误信息
///   - `BadRequest` -- 请求参数验证失败
///   - `Unauthorized` -- 邮箱或密码错误
///   - `ServerError` -- 服务器内部错误
///   - `NotFound` -- 用户未找到
///   - `Forbidden` -- 账户未验证
pub async fn login(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<LoginUserDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let result = app_state
        .db_client
        .get_user(None, None, Some(&body.email), None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::bad_request(
        ErrorMessage::WrongCredentials.to_string(),
    ))?;

    let password_matched = password::compare(&body.password, &user.password)
        .map_err(|_| HttpError::bad_request(ErrorMessage::WrongCredentials.to_string()))?;

    if password_matched {
        let token = token::create_token(
            &user.id.to_string(),
            &app_state.env.jwt_secret.as_bytes(),
            app_state.env.jwt_maxage,
        )
        .map_err(|e| HttpError::server_error(e.to_string()))?;

        let cookie_duration = time::Duration::minutes(app_state.env.jwt_maxage * 60);
        let cookie = Cookie::build(("token", token.clone()))
            .path("/")
            .max_age(cookie_duration)
            .http_only(true)
            .build();

        let response = axum::response::Json(UserLoginResponseDto {
            status: "success".to_string(),
            token,
        });

        let mut headers = HeaderMap::new();

        headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());

        let mut response = response.into_response();
        response.headers_mut().extend(headers);

        Ok(response)
    } else {
        Err(HttpError::bad_request(
            ErrorMessage::WrongCredentials.to_string(),
        ))
    }
}

pub async fn verify_email(
    Query(query_params): Query<VerifyEmailQueryDto>,
    Extension(app_state): Extension<Arc<AppState>>,
) -> Result<impl IntoResponse, HttpError> {
    query_params
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let result = app_state
        .db_client
        .get_user(None, None, None, Some(&query_params.token))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::unauthorized(
        ErrorMessage::InvalidToken.to_string(),
    ))?;

    if let Some(expires_at) = user.token_expires_at {
        if Utc::now() > expires_at {
            return Err(HttpError::bad_request(
                "Verification token has expired".to_string(),
            ))?;
        }
    } else {
        return Err(HttpError::bad_request(
            "Invalid verification token".to_string(),
        ))?;
    }

    app_state
        .db_client
        .verifed_token(&query_params.token)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let send_welcome_email_result = send_welcome_email(&user.email, &user.name).await;

    if let Err(e) = send_welcome_email_result {
        eprintln!("Failed to send welcome email: {}", e);
    }

    let token = token::create_token(
        &user.id.to_string(),
        app_state.env.jwt_secret.as_bytes(),
        app_state.env.jwt_maxage,
    )
    .map_err(|e| HttpError::server_error(e.to_string()))?;

    let cookie_duration = time::Duration::minutes(app_state.env.jwt_maxage * 60);
    let cookie = Cookie::build(("token", token.clone()))
        .path("/")
        .max_age(cookie_duration)
        .http_only(true)
        .build();

    let mut headers = HeaderMap::new();

    headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    let frontend_url = format!("http://localhost:5173/settings");

    let redirect = Redirect::to(&frontend_url);

    let mut response = redirect.into_response();

    response.headers_mut().extend(headers);

    Ok(response)
}

pub async fn forgot_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<ForgotPasswordRequestDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let result = app_state
        .db_client
        .get_user(None, None, Some(&body.email), None)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::bad_request("Email not found!".to_string()))?;

    let verification_token = uuid::Uuid::new_v4().to_string();
    let expires_at = Utc::now() + Duration::minutes(30);

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state
        .db_client
        .add_verifed_token(user_id, &verification_token, expires_at)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let reset_link = format!(
        "http://localhost:5173/reset-password?token={}",
        &verification_token
    );

    let email_sent = send_forgot_password_email(&user.email, &reset_link, &user.name).await;

    if let Err(e) = email_sent {
        eprintln!("Failed to send forgot password email: {}", e);
        return Err(HttpError::server_error("Failed to send email".to_string()));
    }

    let response = Response {
        message: "Password reset link has been sent to your email.".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn reset_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<ResetPasswordRequestDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let result = app_state
        .db_client
        .get_user(None, None, None, Some(&body.token))
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::bad_request(
        "Invalid or expired token".to_string(),
    ))?;

    if let Some(expires_at) = user.token_expires_at {
        if Utc::now() > expires_at {
            return Err(HttpError::bad_request(
                "Verification token has expired".to_string(),
            ))?;
        }
    } else {
        return Err(HttpError::bad_request(
            "Invalid verification token".to_string(),
        ))?;
    }

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let hash_password =
        password::hash(&body.new_password).map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state
        .db_client
        .update_user_password(user_id.clone(), hash_password)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state
        .db_client
        .verifed_token(&body.token)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = Response {
        message: "Password has been successfully reset.".to_string(),
        status: "success",
    };

    Ok(Json(response))
}
