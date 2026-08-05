use crate::error::AppResult;
use crate::guards::AuthUser;
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};

pub async fn dashboard(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<Value>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;

    let month_total: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM issues WHERE created_at >= date_trunc('month', now())",
    )
    .fetch_one(&state.db)
    .await?;
    let all_total: i64 = sqlx::query_scalar("SELECT count(*) FROM issues")
        .fetch_one(&state.db)
        .await?;
    let completed: i64 = sqlx::query_scalar("SELECT count(*) FROM issues WHERE status = '已办结'")
        .fetch_one(&state.db)
        .await?;

    let by_category: Vec<(String, i64)> = sqlx::query_as(
        "SELECT category, count(*) FROM issues GROUP BY category ORDER BY count(*) DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let dept_pending: Vec<(String, i64)> = sqlx::query_as(
        "SELECT d.name, count(i.id)::bigint
         FROM departments d
         LEFT JOIN issues i ON i.department_id = d.id AND i.status IN ('已受理', '部门处理中', '待审核')
         GROUP BY d.name ORDER BY count(i.id) DESC",
    )
    .fetch_all(&state.db)
    .await?;

    let avg_response_hours: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(EXTRACT(EPOCH FROM (accepted_at - created_at)) / 3600.0) FROM issues WHERE accepted_at IS NOT NULL",
    )
    .fetch_one(&state.db)
    .await?;
    let avg_completion_hours: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(EXTRACT(EPOCH FROM (completed_at - created_at)) / 3600.0) FROM issues WHERE completed_at IS NOT NULL",
    )
    .fetch_one(&state.db)
    .await?;

    let satisfaction_avg: Option<f64> = sqlx::query_scalar(
        "SELECT AVG(CASE rating WHEN '满意' THEN 5.0 WHEN '一般' THEN 3.0 WHEN '不满意' THEN 1.0 END)
         FROM issues WHERE rating IS NOT NULL",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(Json(json!({
        "month_total": month_total,
        "all_total": all_total,
        "completion_rate": if all_total == 0 { 0.0 } else { completed as f64 / all_total as f64 },
        "by_category": by_category,
        "department_pending": dept_pending,
        "avg_response_hours": avg_response_hours,
        "avg_completion_hours": avg_completion_hours,
        "satisfaction_avg": satisfaction_avg,
    })))
}
