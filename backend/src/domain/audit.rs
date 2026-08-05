use crate::error::AppResult;
use sqlx::postgres::PgConnection;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn log_audit(
    conn: &mut PgConnection,
    operator_id: Option<Uuid>,
    action: &str,
    target_type: &str,
    target_id: &str,
    before: Option<&str>,
    after: Option<&str>,
    detail: &str,
    ip: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO audit_logs (operator_id, action, target_type, target_id, before, after, detail, ip)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(operator_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(before)
    .bind(after)
    .bind(detail)
    .bind(ip)
    .execute(conn)
    .await?;
    Ok(())
}
