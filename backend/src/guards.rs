use crate::auth::{decode_token, Claims};
use crate::error::{AppError, AppResult};
use crate::permissions::{self, Ctx};
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

    /// 权限节点校验（继承 + 上下文），失败返回 403。
    pub fn check(&self, node: &str, ctx: &Ctx) -> AppResult<()> {
        permissions::check(self.role(), self.id(), self.0.dept, node, ctx)
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
