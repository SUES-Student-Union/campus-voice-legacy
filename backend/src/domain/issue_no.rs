use crate::error::{AppError, AppResult};
use chrono::Utc;
use sqlx::postgres::PgConnection;

/// 生成诉求编号：CV + YYYYMMDD + 4 位序列。
pub async fn next_issue_no(conn: &mut PgConnection) -> AppResult<String> {
    let seq: i64 = sqlx::query_scalar("SELECT nextval('issue_no_seq')")
        .fetch_one(conn)
        .await
        .map_err(|e| AppError::internal(format!("生成编号失败: {e}")))?;
    let now = Utc::now();
    Ok(format!(
        "CV{}{}{}{:04}",
        now.format("%Y"),
        now.format("%m"),
        now.format("%d"),
        seq
    ))
}
