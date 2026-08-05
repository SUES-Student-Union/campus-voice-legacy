pub mod auth;
pub mod config;
pub mod domain;
pub mod error;
pub mod guards;
pub mod handlers;
pub mod models;
pub mod permissions;
pub mod routes;
pub mod seed;
pub mod sso;
pub mod state;

pub use config::AppConfig;
pub use state::AppState;

use anyhow::Context;

pub async fn connect_pool(database_url: &str) -> anyhow::Result<sqlx::PgPool> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await
        .context("连接数据库失败")
}

pub async fn migrate(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::migrate!()
        .run(pool)
        .await
        .context("执行数据库迁移失败")
}

pub async fn seed_admin(pool: &sqlx::PgPool, cfg: &AppConfig) -> anyhow::Result<()> {
    let existing =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM users WHERE username = $1)")
            .bind(&cfg.admin_username)
            .fetch_one(pool)
            .await?;
    if existing {
        return Ok(());
    }
    let hash = auth::hash_password(&cfg.admin_password)
        .map_err(|e| anyhow::anyhow!("口令散列失败: {e}"))?;
    sqlx::query(
        "INSERT INTO users (username, password_hash, role, name) VALUES ($1, $2, 'admin', $3)",
    )
    .bind(&cfg.admin_username)
    .bind(&hash)
    .bind(&cfg.admin_username)
    .execute(pool)
    .await?;
    tracing::info!(username = %cfg.admin_username, "已创建管理员账号（请立即修改密码）");
    Ok(())
}
