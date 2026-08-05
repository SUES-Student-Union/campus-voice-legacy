use crate::error::{AppError, AppResult};
use crate::handlers::push_arg;
use crate::models::{IssueProgress, ProgressOut, PublicCase, PublicCaseOut};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::Deserialize;
use sqlx::postgres::PgArguments;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PublicQuery {
    pub category: Option<String>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

pub async fn list_public(
    State(state): State<AppState>,
    Query(q): Query<PublicQuery>,
) -> AppResult<Json<Vec<PublicCaseOut>>> {
    let mut conditions: Vec<String> = vec!["published_at IS NOT NULL".to_string()];
    let mut args = PgArguments::default();
    let mut idx = 1usize;

    if let Some(ref v) = q.category {
        conditions.push(format!("category = ${idx}"));
        push_arg(&mut args, v.clone())?;
        idx += 1;
    }
    if let Some(ref v) = q.status {
        conditions.push(format!("status = ${idx}"));
        push_arg(&mut args, v.clone())?;
        idx += 1;
    }
    if let Some(ref v) = q.keyword {
        if !v.trim().is_empty() {
            conditions.push(format!("(title ILIKE ${idx} OR summary ILIKE ${idx})"));
            push_arg(&mut args, format!("%{}%", v.trim()))?;
            idx += 1;
        }
    }
    let _ = idx;

    let sql = format!(
        "SELECT * FROM public_cases WHERE {} ORDER BY published_at DESC LIMIT 100",
        conditions.join(" AND ")
    );
    let rows = sqlx::query_as_with::<_, PublicCase, PgArguments>(&sql, args)
        .fetch_all(&state.db)
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(to_out(&state, row).await?);
    }
    Ok(Json(out))
}

pub async fn public_detail(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PublicCaseOut>> {
    let row = sqlx::query_as::<_, PublicCase>(
        "SELECT * FROM public_cases WHERE id = $1 AND published_at IS NOT NULL",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound("公开反馈不存在".into()))?;
    Ok(Json(to_out(&state, row).await?))
}

async fn to_out(state: &AppState, row: PublicCase) -> AppResult<PublicCaseOut> {
    let progress: Vec<IssueProgress> = sqlx::query_as(
        "SELECT * FROM issue_progress WHERE issue_id = $1 ORDER BY created_at ASC, id ASC",
    )
    .bind(row.issue_id)
    .fetch_all(&state.db)
    .await?;
    Ok(PublicCaseOut {
        id: row.id,
        category: row.category,
        campus: row.campus,
        location: row.location,
        title: row.title,
        summary: row.summary,
        reply: row.reply,
        status: row.status,
        submitted_at: row.submitted_at,
        completed_at: row.completed_at,
        progress: progress
            .into_iter()
            .map(|p| ProgressOut {
                status: p.status,
                operator_name: p.operator_name,
                note: p.note,
                created_at: p.created_at,
            })
            .collect(),
    })
}
