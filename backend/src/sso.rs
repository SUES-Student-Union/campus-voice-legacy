//! 学校统一认证适配器（M3）
//!
//! 生产：调用学校统一身份认证（CAS/OAuth，通常经校园网）验证 学号/工号+密码，
//! 返回权威身份（学号/工号、姓名、角色、部门）。
//! 开发：未接入学校认证时使用 mock（见 `dev_mock`），接口契约一致，上线替换实现。

use crate::auth::{ROLE_ADMIN, ROLE_DEPT_STAFF, ROLE_STUDENT, ROLE_WORKER};
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct SsoProfile {
    pub campus_id: String,
    pub name: String,
    pub role: String,
    pub department_code: Option<String>,
}

pub async fn verify(campus_id: &str, password: &str) -> AppResult<SsoProfile> {
    // TODO: 接入学校统一认证后，替换为真实调用（可加 cfg.sso_endpoint / 密钥）。
    dev_mock(campus_id, password)
}

/// 开发 mock：
/// - `admin` -> 管理员
/// - 以 `W` 开头（模拟工号）-> 权益小组
/// - 以 `T` 开头（模拟工号）-> 后勤部门老师
/// - 其余 -> 学生
fn dev_mock(campus_id: &str, password: &str) -> AppResult<SsoProfile> {
    let id = campus_id.trim();
    if id.is_empty() || password.is_empty() {
        return Err(AppError::BadRequest("学号/工号和密码不能为空".into()));
    }
    let (role, department_code): (String, Option<String>) = if id.eq_ignore_ascii_case(ROLE_ADMIN) {
        (ROLE_ADMIN.to_string(), None)
    } else if id.starts_with('W') {
        (ROLE_WORKER.to_string(), None)
    } else if id.starts_with('T') {
        (ROLE_DEPT_STAFF.to_string(), Some("logistics".to_string()))
    } else {
        (ROLE_STUDENT.to_string(), None)
    };
    let kind = if role == ROLE_STUDENT {
        "学生"
    } else {
        "教职工"
    };
    Ok(SsoProfile {
        campus_id: id.to_string(),
        name: format!("{id}（{kind}）"),
        role,
        department_code,
    })
}
