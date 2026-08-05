use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use chrono::Utc;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const ROLE_STUDENT: &str = "student";
pub const ROLE_WORKER: &str = "worker";
pub const ROLE_DEPT_STAFF: &str = "department_staff";
pub const ROLE_ADMIN: &str = "admin";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub name: String,
    pub dept: Option<Uuid>,
    pub iat: usize,
    pub exp: usize,
}

pub fn sign_token(
    cfg: &AppConfig,
    user_id: Uuid,
    role: &str,
    name: &str,
    dept: Option<Uuid>,
) -> AppResult<String> {
    let now = Utc::now().timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        name: name.to_string(),
        dept,
        iat: now,
        exp: now + (cfg.jwt_expires_hours as usize) * 3600,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(cfg.jwt_secret.as_bytes()),
    )
    .map_err(|e| AppError::internal(format!("jwt 签发失败: {e}")))
}

pub fn decode_token(cfg: &AppConfig, token: &str) -> AppResult<Claims> {
    let mut validation = Validation::default();
    validation.validate_exp = true;
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(cfg.jwt_secret.as_bytes()),
        &validation,
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized("登录已过期，请重新登录".into()))
}

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::internal(format!("口令散列失败: {e}")))
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash)
        .ok()
        .and_then(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .ok()
        })
        .is_some()
}

/// 微信登录：配置了 appid/secret 时调 code2session 换取 openid；否则开发降级。
pub async fn wechat_openid(
    cfg: &AppConfig,
    http: &reqwest::Client,
    code: &str,
) -> AppResult<String> {
    if !cfg.wechat_enabled() {
        tracing::info!("未配置微信登录，使用开发降级 openid（dev_student）");
        return Ok("dev_student".to_string());
    }
    let appid = cfg.wechat_appid.as_deref().unwrap_or_default();
    let secret = cfg.wechat_secret.as_deref().unwrap_or_default();
    let url = "https://api.weixin.qq.com/sns/jscode2session";
    let resp: WechatSessionResp = http
        .get(url)
        .query(&[
            ("appid", appid),
            ("secret", secret),
            ("js_code", code),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?
        .json()
        .await?;
    if let Some(code) = resp.errcode {
        return Err(AppError::BadRequest(format!(
            "微信登录失败({code})：{}",
            resp.errmsg.unwrap_or_default()
        )));
    }
    resp.openid
        .ok_or_else(|| AppError::BadRequest("微信登录未返回 openid".into()))
}

#[derive(Deserialize)]
struct WechatSessionResp {
    openid: Option<String>,
    errcode: Option<i64>,
    errmsg: Option<String>,
}
