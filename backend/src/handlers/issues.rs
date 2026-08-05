use crate::domain::status::*;
use crate::domain::{issue_no, notify};
use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::handlers::{build_issue_detail, get_issue};
use crate::models::{
    CreateIssueReq, Issue, IssueDetail, IssueListItem, IssueListRow, RatingReq, SupplementReq,
};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

const KINDS: [&str; 5] = ["意见建议", "问题咨询", "投诉反馈", "设施报修建议", "其他"];
const RATINGS: [&str; 3] = ["满意", "一般", "不满意"];

pub async fn create_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<CreateIssueReq>,
) -> AppResult<Json<IssueDetail>> {
    user.check(
        crate::permissions::ISSUE_SUBMIT,
        &crate::permissions::Ctx::none(),
    )?;
    validate_create(&req)?;

    let category_enabled =
        sqlx::query_scalar::<_, bool>("SELECT enabled FROM categories WHERE name = $1")
            .bind(&req.category)
            .fetch_optional(&state.db)
            .await?
            .unwrap_or(false);
    if !category_enabled {
        return Err(AppError::BadRequest(format!("分类“{}”无效", req.category)));
    }

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    let issue_no_str = issue_no::next_issue_no(&mut tx).await?;
    let contact = req.contact.unwrap_or_default().trim().to_string();
    let contact = if contact.is_empty() {
        "微信联系".to_string()
    } else {
        contact
    };

    let issue = sqlx::query_as::<_, Issue>(
        "INSERT INTO issues
           (issue_no, submitter_id, kind, category, campus, location, title, description,
            public_will, contact, latest_progress, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, '待受理')
         RETURNING *",
    )
    .bind(issue_no_str)
    .bind(user.id())
    .bind(&req.kind)
    .bind(&req.category)
    .bind(&req.campus)
    .bind(&req.location)
    .bind(&req.title)
    .bind(&req.description)
    .bind(req.public_will)
    .bind(&contact)
    .bind("诉求已提交，等待权益工作小组查看。")
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::internal)?;

    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_PENDING,
        Some(user.id()),
        "学生",
        "诉求已提交并生成编号。",
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    Ok(Json(build_issue_detail(&state.db, &issue).await?))
}

pub async fn my_issues(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<IssueListItem>>> {
    user.check(
        crate::permissions::ISSUE_VIEW_OWN,
        &crate::permissions::Ctx::owner(user.id()),
    )?;
    let rows = sqlx::query_as::<_, IssueListRow>(
        "SELECT i.*, d.name AS department_name
         FROM issues i LEFT JOIN departments d ON d.id = i.department_id
         WHERE i.submitter_id = $1 ORDER BY i.created_at DESC",
    )
    .bind(user.id())
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows.into_iter().map(IssueListItem::from).collect()))
}

pub async fn my_issue_detail(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<IssueDetail>> {
    let issue = get_issue(&state.db, id).await?;
    user.check(
        crate::permissions::ISSUE_VIEW_OWN,
        &crate::permissions::Ctx::owner(issue.submitter_id),
    )?;
    Ok(Json(build_issue_detail(&state.db, &issue).await?))
}

pub async fn supplement_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<SupplementReq>,
) -> AppResult<Json<IssueDetail>> {
    let content = req.content.trim();
    if content.is_empty() {
        return Err(AppError::BadRequest("补充内容不能为空".into()));
    }
    let issue = get_issue(&state.db, id).await?;
    user.check(
        crate::permissions::ISSUE_SUPPLEMENT,
        &crate::permissions::Ctx::owner(issue.submitter_id),
    )?;
    if issue.status != STATUS_SUPPLEMENT {
        return Err(AppError::Conflict("当前状态无需补充材料".into()));
    }

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query("UPDATE issues SET status = '已受理', latest_progress = $1, updated_at = now() WHERE id = $2")
        .bind(content)
        .bind(issue.id)
        .execute(&mut *tx)
        .await?;
    notify::push_progress(
        &mut tx,
        issue.id,
        STATUS_ACCEPTED,
        Some(user.id()),
        "学生",
        "已补充信息，等待继续处理。",
    )
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

pub async fn rate_issue(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<RatingReq>,
) -> AppResult<Json<IssueDetail>> {
    if !RATINGS.contains(&req.rating.as_str()) {
        return Err(AppError::BadRequest(
            "评分必须是：满意 / 一般 / 不满意".into(),
        ));
    }
    let issue = get_issue(&state.db, id).await?;
    user.check(
        crate::permissions::ISSUE_RATE,
        &crate::permissions::Ctx::owner(issue.submitter_id),
    )?;
    if issue.status != STATUS_COMPLETED {
        return Err(AppError::Conflict("仅办结的诉求可评价".into()));
    }
    if issue.rating.is_some() {
        return Err(AppError::Conflict("该诉求已评价过".into()));
    }

    let mut tx = state.db.begin().await.map_err(AppError::internal)?;
    sqlx::query(
        "UPDATE issues SET rating = $1, rating_comment = $2, rated_at = now(), updated_at = now() WHERE id = $3",
    )
    .bind(&req.rating)
    .bind(if req.comment.trim().is_empty() { None } else { Some(req.comment.trim().to_string()) })
    .bind(issue.id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await.map_err(AppError::internal)?;

    let updated = get_issue(&state.db, id).await?;
    Ok(Json(build_issue_detail(&state.db, &updated).await?))
}

fn validate_create(req: &CreateIssueReq) -> AppResult<()> {
    if !KINDS.contains(&req.kind.as_str()) {
        return Err(AppError::BadRequest("诉求类型无效".into()));
    }
    if req.campus.trim().is_empty() {
        return Err(AppError::BadRequest("请选择校区".into()));
    }
    if req.location.trim().len() > 60 || req.location.trim().is_empty() {
        return Err(AppError::BadRequest("具体地点必填且不超过 60 字".into()));
    }
    if req.title.trim().is_empty() || req.title.trim().len() > 40 {
        return Err(AppError::BadRequest("标题必填且不超过 40 字".into()));
    }
    if req.description.trim().len() < 10 {
        return Err(AppError::BadRequest("详细描述至少 10 个字".into()));
    }
    Ok(())
}
