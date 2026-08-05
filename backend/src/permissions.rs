//! 权限层（借鉴 LuckPerms：组 + 节点 + 上下文作用域）
//!
//! - **组继承**：admin 继承 worker 继承 student；department_staff 继承 student。
//! - **节点**：每个动作一个权限节点（如 `issue.accept`）。
//! - **上下文作用域**：`Any`（无条件）、`Own`（必须本人诉求）、`Dept`（必须本部门诉求）。
//!
//! 与 LuckPerms 的区别：V1 不做动态节点存储 / 显式否定（`!node`），组与节点为声明式常量。

use crate::auth::{ROLE_ADMIN, ROLE_DEPT_STAFF, ROLE_STUDENT, ROLE_WORKER};
use crate::error::{AppError, AppResult};
use uuid::Uuid;

// ---------- 权限节点 ----------
pub const ISSUE_SUBMIT: &str = "issue.submit";
pub const ISSUE_VIEW_OWN: &str = "issue.view.own";
pub const ISSUE_VIEW_ALL: &str = "issue.view.all";
pub const ISSUE_SUPPLEMENT: &str = "issue.supplement";
pub const ISSUE_RATE: &str = "issue.rate";
pub const ISSUE_ACCEPT: &str = "issue.accept";
pub const ISSUE_ASSIGN: &str = "issue.assign";
pub const ISSUE_REQUEST_SUPPLEMENT: &str = "issue.request_supplement";
pub const ISSUE_CLOSE: &str = "issue.close";
pub const ISSUE_MERGE: &str = "issue.merge";
pub const ISSUE_PROGRESS: &str = "issue.progress";
pub const ISSUE_REPLY: &str = "issue.reply";
pub const REVIEW_SAVE: &str = "review.save";
pub const REVIEW_APPROVE: &str = "review.approve";
pub const REVIEW_REJECT: &str = "review.reject";
pub const AUDIT_VIEW: &str = "audit.view";
pub const DASHBOARD_VIEW: &str = "dashboard.view";
pub const DEPARTMENT_MANAGE: &str = "department.manage";
pub const ATTACHMENT_UPLOAD: &str = "attachment.upload";
pub const ATTACHMENT_DOWNLOAD: &str = "attachment.download";
pub const NOTIFICATION_VIEW: &str = "notification.view";

/// 节点作用域。
#[derive(Clone, Copy, PartialEq)]
pub enum Scope {
    /// 无条件。
    Any,
    /// 仅本人诉求（ctx.owner == 用户 id）。
    Own,
    /// 仅本部门诉求（ctx.dept == 用户部门 id）。
    Dept,
}

/// 请求上下文。
#[derive(Default)]
pub struct Ctx {
    pub owner: Option<Uuid>,
    pub dept: Option<Uuid>,
}

impl Ctx {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn owner(id: Uuid) -> Self {
        Self {
            owner: Some(id),
            dept: None,
        }
    }

    /// 由诉求推导上下文：owner=学生，dept=负责部门。
    pub fn for_issue(issue: &crate::models::Issue) -> Self {
        Self {
            owner: Some(issue.submitter_id),
            dept: issue.department_id,
        }
    }
}

/// 继承链：靠前的组优先级更高。
pub fn inherited_groups(role: &str) -> &'static [&'static str] {
    match role {
        ROLE_ADMIN => &[ROLE_ADMIN, ROLE_WORKER, ROLE_STUDENT],
        ROLE_WORKER => &[ROLE_WORKER, ROLE_STUDENT],
        ROLE_DEPT_STAFF => &[ROLE_DEPT_STAFF, ROLE_STUDENT],
        _ => &[ROLE_STUDENT],
    }
}

/// 组 -> (节点, 作用域)。
fn group_nodes(group: &str) -> &'static [(&'static str, Scope)] {
    match group {
        ROLE_STUDENT => &[
            (ISSUE_SUBMIT, Scope::Any),
            (ISSUE_VIEW_OWN, Scope::Own),
            (ISSUE_SUPPLEMENT, Scope::Own),
            (ISSUE_RATE, Scope::Own),
            (ATTACHMENT_UPLOAD, Scope::Own),
            (ATTACHMENT_DOWNLOAD, Scope::Own),
            (NOTIFICATION_VIEW, Scope::Any),
        ],
        ROLE_DEPT_STAFF => &[
            (ISSUE_VIEW_ALL, Scope::Dept),
            (ISSUE_PROGRESS, Scope::Dept),
            (ISSUE_REPLY, Scope::Dept),
            (ATTACHMENT_UPLOAD, Scope::Dept),
            (ATTACHMENT_DOWNLOAD, Scope::Dept),
        ],
        ROLE_WORKER => &[
            (ISSUE_VIEW_ALL, Scope::Any),
            (ISSUE_ACCEPT, Scope::Any),
            (ISSUE_ASSIGN, Scope::Any),
            (ISSUE_REQUEST_SUPPLEMENT, Scope::Any),
            (ISSUE_CLOSE, Scope::Any),
            (ISSUE_MERGE, Scope::Any),
            (REVIEW_SAVE, Scope::Any),
            (REVIEW_APPROVE, Scope::Any),
            (REVIEW_REJECT, Scope::Any),
            (AUDIT_VIEW, Scope::Any),
            (DASHBOARD_VIEW, Scope::Any),
            (ATTACHMENT_UPLOAD, Scope::Any),
            (ATTACHMENT_DOWNLOAD, Scope::Any),
        ],
        ROLE_ADMIN => &[(DEPARTMENT_MANAGE, Scope::Any)],
        _ => &[],
    }
}

/// 校验用户是否对 `node` 拥有权限；任一继承组授予且上下文满足即放行。
pub fn check(
    role: &str,
    user_id: Uuid,
    user_dept: Option<Uuid>,
    node: &str,
    ctx: &Ctx,
) -> AppResult<()> {
    for group in inherited_groups(role) {
        for (n, scope) in group_nodes(group) {
            if *n == node && scope_satisfied(*scope, ctx, user_id, user_dept) {
                return Ok(());
            }
        }
    }
    Err(AppError::Forbidden("当前账号无权执行该操作".into()))
}

fn scope_satisfied(scope: Scope, ctx: &Ctx, user_id: Uuid, user_dept: Option<Uuid>) -> bool {
    match scope {
        Scope::Any => true,
        Scope::Own => ctx.owner == Some(user_id),
        Scope::Dept => user_dept.is_some() && ctx.dept == user_dept,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(role: &str, uid: Uuid, dept: Option<Uuid>, node: &str, ctx: &Ctx) -> bool {
        check(role, uid, dept, node, ctx).is_ok()
    }

    #[test]
    fn inheritance_grants_parent_permissions() {
        let admin = Uuid::new_v4();
        let ctx = Ctx::none();
        // admin 继承 worker -> 可受理
        assert!(ok(ROLE_ADMIN, admin, None, ISSUE_ACCEPT, &ctx));
        // admin 继承 student -> 可提交
        assert!(ok(ROLE_ADMIN, admin, None, ISSUE_SUBMIT, &ctx));
        // worker 可受理，但不能管理部门
        assert!(ok(ROLE_WORKER, admin, None, ISSUE_ACCEPT, &ctx));
        assert!(!ok(ROLE_WORKER, admin, None, DEPARTMENT_MANAGE, &ctx));
        assert!(ok(ROLE_ADMIN, admin, None, DEPARTMENT_MANAGE, &ctx));
        // 学生不可受理
        assert!(!ok(ROLE_STUDENT, admin, None, ISSUE_ACCEPT, &ctx));
    }

    #[test]
    fn own_scope_requires_owner() {
        let me = Uuid::new_v4();
        let other = Uuid::new_v4();
        assert!(ok(ROLE_STUDENT, me, None, ISSUE_VIEW_OWN, &Ctx::owner(me)));
        assert!(!ok(
            ROLE_STUDENT,
            me,
            None,
            ISSUE_VIEW_OWN,
            &Ctx::owner(other)
        ));
    }

    #[test]
    fn dept_scope_requires_department_match() {
        let dept_a = Uuid::new_v4();
        let dept_b = Uuid::new_v4();
        let staff = Uuid::new_v4();
        assert!(ok(
            ROLE_DEPT_STAFF,
            staff,
            Some(dept_a),
            ISSUE_REPLY,
            &Ctx {
                dept: Some(dept_a),
                ..Ctx::default()
            }
        ));
        assert!(!ok(
            ROLE_DEPT_STAFF,
            staff,
            Some(dept_a),
            ISSUE_REPLY,
            &Ctx {
                dept: Some(dept_b),
                ..Ctx::default()
            }
        ));
        // 部门老师不能回复别的部门，也不能受理
        assert!(!ok(
            ROLE_DEPT_STAFF,
            staff,
            Some(dept_a),
            ISSUE_ACCEPT,
            &Ctx::none()
        ));
        // 未绑定部门则 Dept 作用域不通过
        assert!(!ok(
            ROLE_DEPT_STAFF,
            staff,
            None,
            ISSUE_REPLY,
            &Ctx {
                dept: Some(dept_a),
                ..Ctx::default()
            }
        ));
    }

    #[test]
    fn issue_context_checks_both_scopes() {
        let me = Uuid::new_v4();
        let dept_a = Uuid::new_v4();
        let issue = crate::models::Issue {
            id: Uuid::new_v4(),
            issue_no: "CV202601010001".into(),
            submitter_id: me,
            kind: "问题咨询".into(),
            category: "网络".into(),
            campus: "松江校区".into(),
            location: "教学楼".into(),
            title: "t".into(),
            description: "d".into(),
            status: "部门处理中".into(),
            priority: "中".into(),
            is_public: false,
            public_will: true,
            contact: "微信联系".into(),
            department_id: Some(dept_a),
            assignee_id: None,
            deadline: None,
            latest_progress: String::new(),
            rating: None,
            rating_comment: None,
            rated_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            accepted_at: None,
            completed_at: None,
            closed_at: None,
        };
        let ctx = Ctx::for_issue(&issue);
        // 学生本人：可上传附件
        assert!(ok(ROLE_STUDENT, me, None, ATTACHMENT_UPLOAD, &ctx));
        // 部门老师（本部门）：可上传附件
        assert!(ok(
            ROLE_DEPT_STAFF,
            me,
            Some(dept_a),
            ATTACHMENT_UPLOAD,
            &ctx
        ));
        // 其他学生：不可
        assert!(!ok(
            ROLE_STUDENT,
            Uuid::new_v4(),
            None,
            ATTACHMENT_UPLOAD,
            &ctx
        ));
        // 其他部门老师（非本人、非本部门）：不可
        assert!(!ok(
            ROLE_DEPT_STAFF,
            Uuid::new_v4(),
            Some(Uuid::new_v4()),
            ATTACHMENT_UPLOAD,
            &ctx
        ));
        // 工作人员：任意可
        assert!(ok(ROLE_WORKER, me, None, ATTACHMENT_UPLOAD, &ctx));
    }
}
