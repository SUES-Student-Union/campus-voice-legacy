use crate::domain::status::*;
use crate::domain::{notify, status};
use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::handlers::{build_issue_detail, get_issue, push_arg};
use crate::models::{
    AcceptReq, AssignReq, AuditLog, AuditLogOut, Issue, IssueDetail, MergeReq, NoteReq, User,
};
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::postgres::PgArguments;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub status: Option<String>,
    pub category: Option<String>,
    pub department_id: Option<Uuid>,
    pub priority: Option<String>,
    pub keyword: Option<String>,
}

pub async fn list_issues(
    State(state): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQuery>,
) -> AppResult<Json<Vec<crate::models::IssueListItem>>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;

    let mut conditions: Vec<String> = Vec::new();
    let mut args = PgArguments::default();
    let mut idx = 1usize;

    if let Some(ref v) = q.status {
        if status::ALL_STATUSES.contains(&v.as_str()) {
            conditions.push(format!("status = ${idx}"));
            push_arg(&mut args, v.clone())?;
            idx += 1;
        } else {
            return Err(AppError::BadRequest("状态无效".into()));
        }
    }
    if let Some(ref v) = q.category {
        conditions.push(format!("category = ${idx}"));
        push_arg(&mut args, v.clone())?;
        idx += 1;
    }
    if let Some(v) = q.priority {
        if !["高", "中", "低"].contains(&v.as_str()) {
            return Err(AppError::BadRequest("优先级无效".into()));
        }
        conditions.push(format!("priority = ${idx}"));
        push_arg(&mut args, v)?;
        idx += 1;
    }
    if let Some(v) = q.department_id {
        conditions.push(format!("department_id = ${idx}"));
        push_arg(&mut args, v)?;
        idx += 1;
    }
    if let Some(ref v) = q.keyword {
        if !v.trim().is_empty() {
            conditions.push(format!(
                "(title ILIKE ${idx} OR issue_no ILIKE ${idx} OR location ILIKE ${idx})"
            ));
            push_arg(&mut args, format!("%{}%", v.trim()))?;
            idx += 1;
        }
    }
    let _ = idx;

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", conditions.join(" AND "))
    };
    let sql = format!("SELECT * FROM issues{where_clause} ORDER BY created_at DESC LIMIT 200");
    let rows = sqlx::query_as_with::<_, Issue, PgArguments>(&sql, args)
        .fetch_all(&state.db)
        .await?;
    Ok(Json(
        rows.iter()
            .map(|i| crate::models::IssueListItem {
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

pub async fn admin_issue_detail(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<IssueDetail>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;
    let issue = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &issue).await?))
}

pub async fn accept_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AcceptReq>,
) -> AppResult<Json<IssueDetail>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;
    if !["高", "中", "低"].contains(&req.priority.as_str()) {
        return Err(AppError::BadRequest("优先级无效".into()));
    }
    let issue = get_issue(&state.db, id).await?;
    status::assert_transition(&issue.status, STATUS_ACCEPTED)?;

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query(
        "UPDATE issues SET category = $1, priority = $2, is_public = $3, status = '已受理',
                accepted_at = COALESCE(accepted_at, now()), latest_progress = $4, updated_at = now()
         WHERE id = $5",
    )
    .bind(&req.category)
    .bind(&req.priority)
    .bind(req.is_public)
    .bind("已确认进入处理流程。")
    .bind(issue.id)
    .execute(&mut *tx)
    .await?;

    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_ACCEPTED,
        Some(user.id()),
        &user.0.name,
        "已受理，设置分类与优先级。",
    )
    .await?;
    notify::notify_user(
        &mut tx,
        issue.student_id,
        "诉求已受理",
        &format!("您提交的诉求 {} 已受理，请留意后续进展。", issue.issue_no),
        "issue",
        Some(issue.id),
    )
    .await?;
    crate::domain::audit::log_audit(
        &mut tx,
        Some(user.id()),
        "受理诉求",
        "issue",
        &issue.id.to_string(),
        Some(&issue.status),
        Some(STATUS_ACCEPTED),
        "设置分类与优先级",
        None,
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

pub async fn assign_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<AssignReq>,
) -> AppResult<Json<IssueDetail>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;
    let issue = get_issue(&state.db, id).await?;

    let dept_name = sqlx::query_scalar::<_, String>("SELECT name FROM departments WHERE id = $1")
        .bind(req.department_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::BadRequest("部门不存在".into()))?;

    if let Some(assignee_id) = req.assignee_id {
        let assignee = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(assignee_id)
            .fetch_optional(&state.db)
            .await?
            .ok_or_else(|| AppError::BadRequest("负责人不存在".into()))?;
        if assignee.role != crate::auth::ROLE_DEPT_STAFF
            || assignee.department_id != Some(req.department_id)
        {
            return Err(AppError::BadRequest(
                "负责人必须属于该部门的老师账号".into(),
            ));
        }
    }

    if issue.status != STATUS_PROCESSING {
        status::assert_transition(&issue.status, STATUS_PROCESSING)?;
    }

    let deadline = Utc::now() + Duration::days(7);
    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query(
        "UPDATE issues SET department_id = $1, assignee_id = $2, deadline = $3, status = '部门处理中',
                latest_progress = $4, updated_at = now()
         WHERE id = $5",
    )
    .bind(req.department_id)
    .bind(req.assignee_id)
    .bind(deadline)
    .bind(format!("已转交{dept_name}处理。"))
    .bind(issue.id)
    .execute(&mut *tx)
    .await?;

    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_PROCESSING,
        Some(user.id()),
        &user.0.name,
        &format!("已指派至{dept_name}。"),
    )
    .await?;
    if let Some(assignee_id) = req.assignee_id {
        notify::notify_user(
            &mut tx,
            assignee_id,
            "新分配诉求",
            &format!("诉求 {} 已分配至您的部门，请及时处理。", issue.issue_no),
            "issue",
            Some(issue.id),
        )
        .await?;
    }
    crate::domain::audit::log_audit(
        &mut tx,
        Some(user.id()),
        "指派部门",
        "issue",
        &issue.id.to_string(),
        Some(&issue.status),
        Some(STATUS_PROCESSING),
        &format!("指派至 {dept_name}"),
        None,
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

pub async fn request_supplement(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<NoteReq>,
) -> AppResult<Json<IssueDetail>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;
    let note = req.note.trim();
    if note.is_empty() {
        return Err(AppError::BadRequest("补充说明不能为空".into()));
    }
    let issue = get_issue(&state.db, id).await?;
    status::assert_transition(&issue.status, STATUS_SUPPLEMENT)?;

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query(
        "UPDATE issues SET status = '待补充', latest_progress = $1, updated_at = now() WHERE id = $2",
    )
    .bind(note)
    .bind(issue.id)
    .execute(&mut *tx)
    .await?;
    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_SUPPLEMENT,
        Some(user.id()),
        &user.0.name,
        note,
    )
    .await?;
    notify::notify_user(
        &mut tx,
        issue.student_id,
        "需要补充信息",
        &format!("请补充：{note}"),
        "issue",
        Some(issue.id),
    )
    .await?;
    crate::domain::audit::log_audit(
        &mut tx,
        Some(user.id()),
        "请求补充",
        "issue",
        &issue.id.to_string(),
        Some(&issue.status),
        Some(STATUS_SUPPLEMENT),
        note,
        None,
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

pub async fn close_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<NoteReq>,
) -> AppResult<Json<IssueDetail>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;
    let note = req.note.trim();
    if note.is_empty() {
        return Err(AppError::BadRequest("关闭原因不能为空".into()));
    }
    let issue = get_issue(&state.db, id).await?;
    status::assert_transition(&issue.status, STATUS_CLOSED)?;

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query(
        "UPDATE issues SET status = '已关闭', closed_at = now(), latest_progress = $1, updated_at = now() WHERE id = $2",
    )
    .bind(note)
    .bind(issue.id)
    .execute(&mut *tx)
    .await?;
    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_CLOSED,
        Some(user.id()),
        &user.0.name,
        note,
    )
    .await?;
    notify::notify_user(
        &mut tx,
        issue.student_id,
        "诉求已关闭",
        &format!("您提交的诉求 {} 已关闭：{note}", issue.issue_no),
        "issue",
        Some(issue.id),
    )
    .await?;
    crate::domain::audit::log_audit(
        &mut tx,
        Some(user.id()),
        "关闭诉求",
        "issue",
        &issue.id.to_string(),
        Some(&issue.status),
        Some(STATUS_CLOSED),
        note,
        None,
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

pub async fn merge_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<MergeReq>,
) -> AppResult<Json<IssueDetail>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;
    if req.target_issue_id == id {
        return Err(AppError::BadRequest("不能合并到自身".into()));
    }
    let issue = get_issue(&state.db, id).await?;
    let target = get_issue(&state.db, req.target_issue_id).await?;
    if issue.status == STATUS_CLOSED {
        return Err(AppError::Conflict("该诉求已关闭".into()));
    }
    let note = format!(
        "重复诉求，已合并至 {}。{}",
        target.issue_no,
        req.note.trim()
    );

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query(
        "UPDATE issues SET status = '已关闭', closed_at = now(), latest_progress = $1, updated_at = now() WHERE id = $2",
    )
    .bind(&note)
    .bind(issue.id)
    .execute(&mut *tx)
    .await?;
    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_CLOSED,
        Some(user.id()),
        &user.0.name,
        &note,
    )
    .await?;
    notify::notify_user(
        &mut tx,
        issue.student_id,
        "诉求已合并",
        &format!(
            "您的诉求 {} 与 {} 重复，已合并至后者继续跟进。",
            issue.issue_no, target.issue_no
        ),
        "issue",
        Some(issue.id),
    )
    .await?;
    crate::domain::audit::log_audit(
        &mut tx,
        Some(user.id()),
        "合并诉求",
        "issue",
        &issue.id.to_string(),
        Some(&issue.status),
        Some(STATUS_CLOSED),
        &format!("合并至 {}", target.issue_no),
        None,
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

#[derive(sqlx::FromRow)]
struct AuditRow {
    #[sqlx(flatten)]
    log: AuditLog,
    operator_name: Option<String>,
}

pub async fn list_audit_logs(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<AuditLogOut>>> {
    user.require_role(&[crate::auth::ROLE_WORKER, crate::auth::ROLE_ADMIN])?;
    let rows = sqlx::query_as::<_, AuditRow>(
        "SELECT a.*, u.name AS operator_name
         FROM audit_logs a LEFT JOIN users u ON u.id = a.operator_id
         ORDER BY a.created_at DESC LIMIT 200",
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|r| AuditLogOut {
                id: r.log.id,
                operator_name: r.operator_name,
                action: r.log.action,
                target_type: r.log.target_type,
                target_id: r.log.target_id,
                before: r.log.before,
                after: r.log.after,
                detail: r.log.detail,
                ip: r.log.ip,
                created_at: r.log.created_at,
            })
            .collect(),
    ))
}
