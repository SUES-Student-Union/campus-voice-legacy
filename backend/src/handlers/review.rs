use crate::domain::status::*;
use crate::domain::{audit, notify};
use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::handlers::{build_issue_detail, get_issue};
use crate::models::{IssueDetail, PublicCase, ReviewReq};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

pub async fn save_review(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<ReviewReq>,
) -> AppResult<Json<IssueDetail>> {
    user.check(
        crate::permissions::REVIEW_SAVE,
        &crate::permissions::Ctx::none(),
    )?;
    let issue = get_issue(&state.db, id).await?;

    let title = req.title.trim();
    let location = req.location.trim();
    let summary = req.summary.trim();
    let reply = req.reply.trim();

    match req.action.as_str() {
        "draft" => {
            if title.is_empty() || location.is_empty() || summary.is_empty() || reply.is_empty() {
                return Err(AppError::BadRequest(
                    "草稿需填写标题、地点、摘要与回复".into(),
                ));
            }
            upsert_public_case(&state, &issue, title, location, summary, reply, None).await?;
            let mut conn = state.db.acquire().await.map_err(AppError::internal)?;
            audit::log_audit(
                &mut conn,
                Some(user.id()),
                "保存草稿",
                "issue",
                &issue.id.to_string(),
                None,
                None,
                "保存脱敏公开稿草稿",
                None,
            )
            .await?;
            Ok(Json(build_issue_detail(&state.db, &issue).await?))
        }
        "approve" => {
            if issue.status != STATUS_REVIEW {
                return Err(AppError::Conflict("仅在“待审核”状态可审核通过".into()));
            }
            if title.is_empty() || location.is_empty() || summary.is_empty() || reply.is_empty() {
                return Err(AppError::BadRequest("公开稿字段不能为空".into()));
            }
            let completed_at = issue.completed_at.or(Some(chrono::Utc::now()));
            upsert_public_case(
                &state,
                &issue,
                title,
                location,
                summary,
                reply,
                completed_at,
            )
            .await?;

            let mut tx = state.db.begin().await.map_err(AppError::internal)?;
            sqlx::query(
                "UPDATE issues SET status = '已办结', is_public = true, completed_at = COALESCE(completed_at, now()),
                        latest_progress = $1, updated_at = now()
                 WHERE id = $2",
            )
            .bind("办理结果已反馈学生并通过脱敏审核公开。")
            .bind(issue.id)
            .execute(&mut *tx)
            .await?;
            notify::push_progress(
                &mut tx,
                issue.id,
                STATUS_COMPLETED,
                Some(user.id()),
                &user.0.name,
                "脱敏审核通过并公开，诉求办结。",
            )
            .await?;
            notify::notify_user(
                &mut tx,
                issue.submitter_id,
                "诉求已办结",
                &format!("您提交的诉求 {} 已办结，可查看办理结果。", issue.issue_no),
                "issue",
                Some(issue.id),
            )
            .await?;
            audit::log_audit(
                &mut tx,
                Some(user.id()),
                "审核公开",
                "issue",
                &issue.id.to_string(),
                Some(&issue.status),
                Some(STATUS_COMPLETED),
                "脱敏审核通过并公开",
                None,
            )
            .await?;
            tx.commit().await.map_err(AppError::internal)?;

            let updated = get_issue(&state.db, id).await?;
            Ok(Json(build_issue_detail(&state.db, &updated).await?))
        }
        "reject" => {
            if issue.status != STATUS_REVIEW {
                return Err(AppError::Conflict("仅在“待审核”状态可打回".into()));
            }
            let note = req.note.trim();
            if note.is_empty() {
                return Err(AppError::BadRequest("打回原因不能为空".into()));
            }
            let mut tx = state.db.begin().await.map_err(AppError::internal)?;
            sqlx::query(
                "UPDATE issues SET status = '部门处理中', latest_progress = $1, updated_at = now() WHERE id = $2",
            )
            .bind(format!("审核未通过，请补充：{note}"))
            .bind(issue.id)
            .execute(&mut *tx)
            .await?;
            notify::push_progress(
                &mut tx,
                issue.id,
                STATUS_PROCESSING,
                Some(user.id()),
                &user.0.name,
                &format!("审核打回：{note}"),
            )
            .await?;
            audit::log_audit(
                &mut tx,
                Some(user.id()),
                "审核打回",
                "issue",
                &issue.id.to_string(),
                Some(&issue.status),
                Some(STATUS_PROCESSING),
                note,
                None,
            )
            .await?;
            tx.commit().await.map_err(AppError::internal)?;

            let updated = get_issue(&state.db, id).await?;
            Ok(Json(build_issue_detail(&state.db, &updated).await?))
        }
        _ => Err(AppError::BadRequest(
            "action 必须是 draft / approve / reject".into(),
        )),
    }
}

async fn upsert_public_case(
    state: &AppState,
    issue: &crate::models::Issue,
    title: &str,
    location: &str,
    summary: &str,
    reply: &str,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
) -> AppResult<PublicCase> {
    sqlx::query_as::<_, PublicCase>(
        "INSERT INTO public_cases
           (issue_id, category, campus, location, title, summary, reply, status, submitted_at, completed_at, published_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
         ON CONFLICT (issue_id) DO UPDATE SET
           location = EXCLUDED.location,
           title = EXCLUDED.title,
           summary = EXCLUDED.summary,
           reply = EXCLUDED.reply,
           status = EXCLUDED.status,
           completed_at = EXCLUDED.completed_at,
           published_at = EXCLUDED.published_at,
           updated_at = now()
         RETURNING *",
    )
    .bind(issue.id)
    .bind(&issue.category)
    .bind(&issue.campus)
    .bind(location)
    .bind(title)
    .bind(summary)
    .bind(reply)
    .bind(&issue.status)
    .bind(issue.created_at)
    .bind(completed_at)
    .bind(completed_at.map(|_| chrono::Utc::now()))
    .fetch_one(&state.db)
    .await
    .map_err(AppError::internal)
}
