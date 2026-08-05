use crate::error::AppResult;
use sqlx::postgres::PgConnection;
use uuid::Uuid;

pub async fn notify_user(
    conn: &mut PgConnection,
    user_id: Uuid,
    title: &str,
    content: &str,
    kind: &str,
    issue_id: Option<Uuid>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO notifications (user_id, title, content, kind, issue_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(title)
    .bind(content)
    .bind(kind)
    .bind(issue_id)
    .execute(conn)
    .await?;
    Ok(())
}

/// 写入一条进度时间线记录。
pub async fn push_progress(
    conn: &mut PgConnection,
    issue_id: Uuid,
    status: &str,
    operator_id: Option<Uuid>,
    operator_name: &str,
    note: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO issue_progress (issue_id, status, operator_id, operator_name, note)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(issue_id)
    .bind(status)
    .bind(operator_id)
    .bind(operator_name)
    .bind(note)
    .execute(conn)
    .await?;
    Ok(())
}
