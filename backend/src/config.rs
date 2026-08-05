use anyhow::Context;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub server_addr: String,
    pub jwt_secret: String,
    pub jwt_expires_hours: i64,
    pub wechat_appid: Option<String>,
    pub wechat_secret: Option<String>,
    pub admin_username: String,
    pub admin_password: String,
    pub upload_dir: PathBuf,
    pub max_upload_bytes: u64,
    pub max_attachments: usize,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let jwt_secret = env("JWT_SECRET")?;
        if jwt_secret.len() < 32 {
            anyhow::bail!("JWT_SECRET 长度必须 >= 32 字符");
        }

        let admin_password = env("ADMIN_PASSWORD")?;
        if admin_password == "admin123" {
            tracing::warn!("ADMIN_PASSWORD 仍为默认值，请尽快修改");
        }

        Ok(Self {
            database_url: env("DATABASE_URL")?,
            server_addr: env("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            jwt_secret,
            jwt_expires_hours: env("JWT_EXPIRES_HOURS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(168),
            wechat_appid: optional_env("WECHAT_APPID"),
            wechat_secret: optional_env("WECHAT_SECRET"),
            admin_username: env("ADMIN_USERNAME").unwrap_or_else(|_| "admin".into()),
            admin_password,
            upload_dir: PathBuf::from(env("UPLOAD_DIR").unwrap_or_else(|_| "uploads".into())),
            max_upload_bytes: 5 * 1024 * 1024,
            max_attachments: 3,
        })
    }

    pub fn wechat_enabled(&self) -> bool {
        self.wechat_appid.as_deref().is_some_and(|v| !v.is_empty())
            && self.wechat_secret.as_deref().is_some_and(|v| !v.is_empty())
    }
}

fn env(key: &str) -> anyhow::Result<String> {
    std::env::var(key).with_context(|| format!("缺少环境变量 {key}（参考 .env.example）"))
}

fn optional_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.is_empty())
}
