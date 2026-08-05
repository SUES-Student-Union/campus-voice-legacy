use crate::auth::ROLE_DEPT_STAFF;
use crate::domain::status::*;
use crate::domain::{audit, notify};
use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::handlers::{build_issue_detail, get_issue};
use crate::models::{Issue, IssueDetail, IssueListItem, NoteReq, ReplyReq};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

pub async fn dept_issues(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<IssueListItem>>> {
    user.require_role(&[ROLE_DEPT_STAFF])?;

    let dept_id =
        sqlx::query_scalar::<_, Option<Uuid>>("SELECT department_id FROM users WHERE id = $1")
            .bind(user.id())
            .fetch_optional(&state.db)
            .await?
            .flatten()
            .ok_or_else(|| AppError::Forbidden("当前账号未绑定部门".into()))?;

    let rows = sqlx::query_as::<_, Issue>(
        "SELECT * FROM issues WHERE department_id = $1 AND status NOT IN ('已办结', '已关闭')
         ORDER BY created_at DESC",
    )
    .bind(dept_id)
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.iter()
            .map(|i| IssueListItem {
                id: i.id,
                issue_no: i.issue_no.clone(),
                title: i.title.clone(),
                category: i.category.clone(),
                status: i.status.clone(),
                latest_progress: i.latest_progress.clone(),
                updated_at: i.updated_at,
            })
            .collect(),
    ))
}

async fn own_dept_issue(
    state: &AppState,
    user: &AuthUser,
    id: Uuid,
) -> AppResult<crate::models::Issue> {
    let issue = get_issue(&state.db, id).await?;
    let dept_id =
        sqlx::query_scalar::<_, Option<Uuid>>("SELECT department_id FROM users WHERE id = $1")
            .bind(user.id())
            .fetch_optional(&state.db)
            .await?
            .flatten()
            .ok_or_else(|| AppError::Forbidden("当前账号未绑定部门".into()))?;
    if issue.department_id != Some(dept_id) {
        return Err(AppError::Forbidden("只能操作分配给本部门的诉求".into()));
    }
    Ok(issue)
}

pub async fn update_progress(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<NoteReq>,
) -> AppResult<Json<IssueDetail>> {
    user.require_role(&[ROLE_DEPT_STAFF])?;
    let note = req.note.trim();
    if note.is_empty() {
        return Err(AppError::BadRequest("进展说明不能为空".into()));
    }
    let issue = own_dept_issue(&state, &user, id).await?;
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
    user.require_role(&[ROLE_DEPT_STAFF])?;
    let content = req.content.trim();
    if content.is_empty() {
        return Err(AppError::BadRequest("回复内容不能为空".into()));
    }
    let issue = own_dept_issue(&state, &user, id).await?;
    crate::domain::status::assert_transition(&issue.status, STATUS_REVIEW)?;

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    let dept_id =
        sqlx::query_scalar::<_, Option<Uuid>>("SELECT department_id FROM users WHERE id = $1")
            .bind(user.id())
            .fetch_one(&mut *tx)
            .await?
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
        issue.student_id,
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
