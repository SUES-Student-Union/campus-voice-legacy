use crate::config::AppConfig;
use sqlx::postgres::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cfg: Arc<AppConfig>,
    pub http: reqwest::Client,
}
