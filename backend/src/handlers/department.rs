use crate::domain::status::*;
use crate::domain::{audit, notify};
use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::handlers::{build_issue_detail, get_issue};
use crate::models::{IssueDetail, IssueListItem, IssueListRow, NoteReq, ReplyReq};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

pub async fn dept_issues(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<IssueListItem>>> {
    let dept_id = user
        .0
        .dept
        .ok_or_else(|| AppError::Forbidden("当前账号未绑定部门".into()))?;
    user.check(
        crate::permissions::ISSUE_VIEW_ALL,
        &crate::permissions::Ctx {
            dept: Some(dept_id),
            ..Default::default()
        },
    )?;

    let rows = sqlx::query_as::<_, IssueListRow>(
        "SELECT i.*, d.name AS department_name
         FROM issues i LEFT JOIN departments d ON d.id = i.department_id
         WHERE i.department_id = $1 AND i.status NOT IN ('已办结', '已关闭')
         ORDER BY i.created_at DESC",
    )
    .bind(dept_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows.into_iter().map(IssueListItem::from).collect()))
}

pub async fn dept_issue_detail(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<crate::models::AdminIssueDetail>> {
    let issue = get_issue(&state.db, id).await?;
    user.check(
        crate::permissions::ISSUE_VIEW_ALL,
        &crate::permissions::Ctx::for_issue(&issue),
    )?;
    Ok(Json(
        crate::handlers::build_admin_issue_detail(&state.db, &issue).await?,
    ))
}

pub async fn update_progress(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<NoteReq>,
) -> AppResult<Json<IssueDetail>> {
    let note = req.note.trim();
    if note.is_empty() {
        return Err(AppError::BadRequest("进展说明不能为空".into()));
    }
    let issue = get_issue(&state.db, id).await?;
    user.check(
        crate::permissions::ISSUE_PROGRESS,
        &crate::permissions::Ctx::for_issue(&issue),
    )?;
    if issue.status != STATUS_PROCESSING {
        return Err(AppError::Conflict("仅在“部门处理中”可更新进展".into()));
    }

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query("UPDATE issues SET latest_progress = $1, updated_at = now() WHERE id = $2")
        .bind(note)
        .bind(issue.id)
        .execute(&mut *tx)
        .await?;
    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_PROCESSING,
        Some(user.id()),
        &user.0.name,
        note,
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

pub async fn reply_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ReplyReq>,
) -> AppResult<Json<IssueDetail>> {
    let content = req.content.trim();
    if content.is_empty() {
        return Err(AppError::BadRequest("回复内容不能为空".into()));
    }
    let issue = get_issue(&state.db, id).await?;
    user.check(
        crate::permissions::ISSUE_REPLY,
        &crate::permissions::Ctx::for_issue(&issue),
    )?;
    crate::domain::status::assert_transition(&issue.status, STATUS_REVIEW)?;

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    let dept_id = user
        .0
        .dept
        .ok_or_else(|| AppError::Forbidden("当前账号未绑定部门".into()))?;

    sqlx::query(
        "UPDATE issues SET status = '待审核', latest_progress = $1, updated_at = now() WHERE id = $2",
    )
    .bind("部门已提交正式回复，等待权益工作小组审核。")
    .bind(issue.id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO replies (issue_id, department_id, author_id, content) VALUES ($1, $2, $3, $4)",
    )
    .bind(issue.id)
    .bind(dept_id)
    .bind(user.id())
    .bind(content)
    .execute(&mut *tx)
    .await?;

    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_REVIEW,
        Some(user.id()),
        &user.0.name,
        "已提交正式回复，申请办结。",
    )
    .await?;
    notify::notify_user(
        &mut tx,
        issue.submitter_id,
        "部门已回复",
        &format!(
            "您提交的诉求 {} 已收到部门正式回复，正在等待审核。",
            issue.issue_no
        ),
        "issue",
        Some(issue.id),
    )
    .await?;
    audit::log_audit(
        &mut tx,
        Some(user.id()),
        "提交回复",
        "issue",
        &issue.id.to_string(),
        Some(&issue.status),
        Some(STATUS_REVIEW),
        "提交正式回复",
        None,
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}
