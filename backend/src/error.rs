use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("参数错误：{0}")]
    BadRequest(String),
    #[error("未认证或登录已过期")]
    Unauthorized(String),
    #[error("无权限：{0}")]
    Forbidden(String),
    #[error("资源不存在：{0}")]
    NotFound(String),
    #[error("冲突：{0}")]
    Conflict(String),
    #[error("文件过大或类型不支持")]
    PayloadTooLarge(String),
    #[error("内部错误：{0}")]
    Internal(String),
}

impl AppError {
    pub fn internal<E: std::fmt::Display>(err: E) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::Unauthorized(m) => (StatusCode::UNAUTHORIZED, m.clone()),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::PayloadTooLarge(m) => (StatusCode::PAYLOAD_TOO_LARGE, m.clone()),
            AppError::Internal(m) => {
                tracing::error!(error = %m, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "服务器内部错误".into())
            }
        };
        (status, Json(json!({ "error": { "message": message } }))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => AppError::NotFound("记录不存在".into()),
            other => {
                tracing::error!(error = %other, "sqlx error");
                AppError::Internal(other.to_string())
            }
        }
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!(error = %err, "wechat request error");
        AppError::Internal("外部服务调用失败".into())
    }
}

pub type AppResult<T> = Result<T, AppError>;
