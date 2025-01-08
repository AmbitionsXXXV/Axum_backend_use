use axum::{
    extract::Request,
    http::{Method, StatusCode, Uri},
    middleware::Next,
    response::Response,
};
use std::time::Instant;

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
