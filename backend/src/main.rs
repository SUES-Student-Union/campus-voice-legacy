use anyhow::Context;
use campus_voice_backend::{connect_pool, migrate, routes, seed_admin, state::AppState};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info".into()),
        )
        .init();

    let cfg = campus_voice_backend::AppConfig::from_env()?;
    tracing::info!(addr = %cfg.server_addr, "starting campus-voice backend");

    let pool = connect_pool(&cfg.database_url).await?;
    migrate(&pool).await?;
    tokio::fs::create_dir_all(&cfg.upload_dir)
        .await
        .context("创建上传目录失败")?;
    seed_admin(&pool, &cfg).await?;

    let state = AppState {
        db: pool,
        cfg: Arc::new(cfg),
        http: reqwest::Client::new(),
    };

    let app = routes::build_router(state.clone());
    let listener = tokio::net::TcpListener::bind(&state.cfg.server_addr)
        .await
        .context("监听端口失败")?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await.context("服务启动失败")?;
    Ok(())
}
