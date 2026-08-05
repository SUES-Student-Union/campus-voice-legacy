pub mod admin;
pub mod attachments;
pub mod auth;
pub mod department;
pub mod departments;
pub mod issues;
pub mod notifications;
pub mod public;
pub mod review;
pub mod stats;

use crate::error::{AppError, AppResult};
use crate::models::{
    AdminIssueDetail, Attachment, Issue, IssueDetail, IssueProgress, ProgressOut, Reply, ReplyOut,
    ReviewDraftOut, User,
};
use sqlx::postgres::PgArguments;
use sqlx::{Arguments, PgPool};
use uuid::Uuid;

/// 动态查询参数绑定；类型均为可枚举的简单值，绑定失败视为内部错误。
pub fn push_arg<T>(args: &mut PgArguments, value: T) -> AppResult<()>
where
    T: 'static + sqlx::Encode<'static, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    args.add(value).map_err(AppError::internal)
}

pub async fn get_issue(pool: &PgPool, id: Uuid) -> AppResult<Issue> {
    sqlx::query_as::<_, Issue>("SELECT * FROM issues WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound("诉求不存在".into()))
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> AppResult<User> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|_| AppError::NotFound("用户不存在".into()))
}

pub async fn get_department_name(pool: &PgPool, id: Uuid) -> Option<String> {
    sqlx::query_scalar::<_, String>("SELECT name FROM departments WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

pub async fn build_issue_detail(pool: &PgPool, issue: &Issue) -> AppResult<IssueDetail> {
    let progress: Vec<IssueProgress> = sqlx::query_as(
        "SELECT * FROM issue_progress WHERE issue_id = $1 ORDER BY created_at ASC, id ASC",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await?;

    let replies: Vec<Reply> =
        sqlx::query_as("SELECT * FROM replies WHERE issue_id = $1 ORDER BY created_at ASC, id ASC")
            .bind(issue.id)
            .fetch_all(pool)
            .await?;

    let attachments: Vec<Attachment> = sqlx::query_as(
        "SELECT * FROM attachments WHERE issue_id = $1 ORDER BY created_at ASC, id ASC",
    )
    .bind(issue.id)
    .fetch_all(pool)
    .await?;

    let mut reply_out = Vec::with_capacity(replies.len());
    for r in replies {
        let department_name = match r.department_id {
            Some(did) => get_department_name(pool, did).await,
            None => None,
        };
        reply_out.push(ReplyOut {
            id: r.id,
            department_name,
            content: r.content,
            created_at: r.created_at,
        });
    }

    Ok(IssueDetail {
        id: issue.id,
        issue_no: issue.issue_no.clone(),
        kind: issue.kind.clone(),
        category: issue.category.clone(),
        campus: issue.campus.clone(),
        location: issue.location.clone(),
        title: issue.title.clone(),
        description: issue.description.clone(),
        status: issue.status.clone(),
        priority: issue.priority.clone(),
        is_public: issue.is_public,
        public_will: issue.public_will,
        contact: issue.contact.clone(),
        department_name: match issue.department_id {
            Some(did) => get_department_name(pool, did).await,
            None => None,
        },
        latest_progress: issue.latest_progress.clone(),
        rating: issue.rating.clone(),
        created_at: issue.created_at,
        updated_at: issue.updated_at,
        progress: progress
            .into_iter()
            .map(|p| ProgressOut {
                status: p.status,
                operator_name: p.operator_name,
                note: p.note,
                created_at: p.created_at,
            })
            .collect(),
        replies: reply_out,
        attachments: attachments
            .into_iter()
            .map(|a| crate::models::AttachmentOut {
                id: a.id,
                original_name: a.original_name,
                size: a.size,
                mime: a.mime,
                created_at: a.created_at,
            })
            .collect(),
    })
}

pub async fn build_admin_issue_detail(pool: &PgPool, issue: &Issue) -> AppResult<AdminIssueDetail> {
    let base = build_issue_detail(pool, issue).await?;

    let student_name = sqlx::query_scalar::<_, String>("SELECT name FROM users WHERE id = $1")
        .bind(issue.submitter_id)
        .fetch_optional(pool)
        .await?;

    let assignee_name = match issue.assignee_id {
        Some(aid) => {
            sqlx::query_scalar::<_, String>("SELECT name FROM users WHERE id = $1")
                .bind(aid)
                .fetch_optional(pool)
                .await?
        }
        None => None,
    };

    #[derive(sqlx::FromRow)]
    struct DraftRow {
        title: String,
        location: String,
        summary: String,
        reply: String,
        published_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let draft = sqlx::query_as::<_, DraftRow>(
        "SELECT title, location, summary, reply, published_at FROM public_cases WHERE issue_id = $1",
    )
    .bind(issue.id)
    .fetch_optional(pool)
    .await?;

    let (review_draft, review_published) = match draft {
        Some(d) => (
            Some(ReviewDraftOut {
                title: d.title,
                location: d.location,
                summary: d.summary,
                reply: d.reply,
            }),
            d.published_at.is_some(),
        ),
        None => (None, false),
    };

    Ok(AdminIssueDetail {
        base,
        student_name,
        deadline: issue.deadline,
        assignee_name,
        review_draft,
        review_published,
    })
}
