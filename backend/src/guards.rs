use crate::auth::{decode_token, Claims, ROLE_ADMIN, ROLE_WORKER};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

/// 从 Authorization: Bearer <token> 提取并校验登录态。
#[derive(Debug, Clone)]
pub struct AuthUser(pub Claims);

impl AuthUser {
    pub fn id(&self) -> uuid::Uuid {
        self.0.sub.parse().unwrap_or_else(|_| uuid::Uuid::nil())
    }

    pub fn role(&self) -> &str {
        &self.0.role
    }

    pub fn is_worker_or_admin(&self) -> bool {
        matches!(self.role(), ROLE_WORKER | ROLE_ADMIN)
    }

    pub fn require_role(&self, roles: &[&str]) -> AppResult<()> {
        if roles.iter().any(|r| *r == self.role()) {
            Ok(())
        } else {
            Err(AppError::Forbidden("当前角色无权执行该操作".into()))
        }
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> AppResult<Self> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("未登录".into()))?;
        let token = header
            .strip_prefix("Bearer ")
            .or_else(|| header.strip_prefix("bearer "))
            .ok_or_else(|| AppError::Unauthorized("认证头格式错误".into()))?;
        let claims = decode_token(&state.cfg, token)?;
        Ok(AuthUser(claims))
    }
}
