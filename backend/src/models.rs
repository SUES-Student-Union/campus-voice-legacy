// 数据定义文件：FromRow 实体需覆盖全列，部分字段留给后续管理端/小程序接入使用。
#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ---------- 实体（FromRow） ----------

#[derive(Clone, Debug, FromRow)]
pub struct User {
    pub id: Uuid,
    pub campus_id: Option<String>,
    pub openid: Option<String>,
    pub username: Option<String>,
    pub password_hash: Option<String>,
    pub role: String,
    pub name: String,
    pub department_id: Option<Uuid>,
    pub phone_masked: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct Department {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct Category {
    pub id: Uuid,
    pub name: String,
    pub sort: i32,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Clone, Debug, FromRow)]
pub struct Issue {
    pub id: Uuid,
    pub issue_no: String,
    pub submitter_id: Uuid,
    pub kind: String,
    pub category: String,
    pub campus: String,
    pub location: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub is_public: bool,
    pub public_will: bool,
    pub contact: String,
    pub department_id: Option<Uuid>,
    pub assignee_id: Option<Uuid>,
    pub deadline: Option<DateTime<Utc>>,
    pub latest_progress: String,
    pub rating: Option<String>,
    pub rating_comment: Option<String>,
    pub rated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, FromRow)]
pub struct IssueProgress {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub status: String,
    pub operator_id: Option<Uuid>,
    pub operator_name: String,
    pub note: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct Reply {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub department_id: Option<Uuid>,
    pub author_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub issue_id: Option<Uuid>,
    pub uploader_id: Uuid,
    pub original_name: String,
    pub stored_name: String,
    pub size: i64,
    pub mime: String,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub content: String,
    pub kind: String,
    pub read: bool,
    pub issue_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub operator_id: Option<Uuid>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub detail: String,
    pub ip: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, FromRow)]
pub struct PublicCase {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub category: String,
    pub campus: String,
    pub location: String,
    pub title: String,
    pub summary: String,
    pub reply: String,
    pub status: String,
    pub submitted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ---------- 输出 DTO ----------

#[derive(Serialize)]
pub struct UserOut {
    pub id: Uuid,
    pub role: String,
    pub name: String,
    pub campus_id: Option<String>,
    pub department_id: Option<Uuid>,
    pub phone_masked: Option<String>,
}

impl From<&User> for UserOut {
    fn from(u: &User) -> Self {
        Self {
            id: u.id,
            role: u.role.clone(),
            name: u.name.clone(),
            campus_id: u.campus_id.clone(),
            department_id: u.department_id,
            phone_masked: u.phone_masked.clone(),
        }
    }
}

#[derive(Serialize)]
pub struct IssueListItem {
    pub id: Uuid,
    pub issue_no: String,
    pub title: String,
    pub category: String,
    pub campus: String,
    pub department_name: Option<String>,
    pub status: String,
    pub priority: String,
    pub latest_progress: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 诉求列表行：Issue + 部门名（联表查询用）。
#[derive(Clone, Debug, FromRow)]
pub struct IssueListRow {
    #[sqlx(flatten)]
    pub issue: Issue,
    pub department_name: Option<String>,
}

impl From<IssueListRow> for IssueListItem {
    fn from(row: IssueListRow) -> Self {
        Self {
            id: row.issue.id,
            issue_no: row.issue.issue_no,
            title: row.issue.title,
            category: row.issue.category,
            campus: row.issue.campus,
            department_name: row.department_name,
            status: row.issue.status,
            priority: row.issue.priority,
            latest_progress: row.issue.latest_progress,
            created_at: row.issue.created_at,
            updated_at: row.issue.updated_at,
        }
    }
}

#[derive(Serialize)]
pub struct ProgressOut {
    pub status: String,
    pub operator_name: String,
    pub note: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct ReplyOut {
    pub id: Uuid,
    pub department_name: Option<String>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AttachmentOut {
    pub id: Uuid,
    pub original_name: String,
    pub size: i64,
    pub mime: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct IssueDetail {
    pub id: Uuid,
    pub issue_no: String,
    pub kind: String,
    pub category: String,
    pub campus: String,
    pub location: String,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub is_public: bool,
    pub public_will: bool,
    pub contact: String,
    pub department_name: Option<String>,
    pub latest_progress: String,
    pub rating: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub progress: Vec<ProgressOut>,
    pub replies: Vec<ReplyOut>,
    pub attachments: Vec<AttachmentOut>,
}

#[derive(Serialize)]
pub struct PublicCaseOut {
    pub id: Uuid,
    pub category: String,
    pub campus: String,
    pub location: String,
    pub title: String,
    pub summary: String,
    pub reply: String,
    pub status: String,
    pub department_name: Option<String>,
    pub submitted_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: Vec<ProgressOut>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserOut,
}

#[derive(Serialize)]
pub struct ReviewDraftOut {
    pub title: String,
    pub location: String,
    pub summary: String,
    pub reply: String,
}

/// 管理端/部门端诉求详情：在 IssueDetail 基础上补充学生、负责人、时限与审核草稿信息。
#[derive(Serialize)]
pub struct AdminIssueDetail {
    #[serde(flatten)]
    pub base: IssueDetail,
    pub student_name: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub assignee_name: Option<String>,
    pub review_draft: Option<ReviewDraftOut>,
    pub review_published: bool,
}

#[derive(Serialize)]
pub struct NotificationOut {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub kind: String,
    pub read: bool,
    pub issue_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct AuditLogOut {
    pub id: Uuid,
    pub operator_name: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub before: Option<String>,
    pub after: Option<String>,
    pub detail: String,
    pub ip: Option<String>,
    pub created_at: DateTime<Utc>,
}

// ---------- 输入 DTO ----------

#[derive(Deserialize)]
pub struct WechatLoginReq {
    pub code: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SsoLoginReq {
    pub campus_id: String,
    pub password: String,
    /// 微信 code，用于把 openid 绑定到校园账号。
    pub code: String,
}

#[derive(Deserialize)]
pub struct CreateIssueReq {
    pub kind: String,
    pub category: String,
    pub campus: String,
    pub location: String,
    pub title: String,
    pub description: String,
    pub public_will: bool,
    pub contact: Option<String>,
}

#[derive(Deserialize)]
pub struct SupplementReq {
    pub content: String,
}

#[derive(Deserialize)]
pub struct RatingReq {
    pub rating: String,
    #[serde(default)]
    pub comment: String,
}

#[derive(Deserialize)]
pub struct AcceptReq {
    pub category: String,
    pub priority: String,
    pub is_public: bool,
}

#[derive(Deserialize)]
pub struct AssignReq {
    pub department_id: Uuid,
    pub assignee_id: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct NoteReq {
    pub note: String,
}

#[derive(Deserialize)]
pub struct MergeReq {
    pub target_issue_id: Uuid,
    pub note: String,
}

#[derive(Deserialize)]
pub struct ReplyReq {
    pub content: String,
}

#[derive(Deserialize)]
pub struct ReviewReq {
    pub action: String, // draft | approve | reject
    pub title: String,
    pub location: String,
    pub summary: String,
    pub reply: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Deserialize)]
pub struct CreateDepartmentReq {
    pub name: String,
    pub code: String,
    pub description: Option<String>,
}
