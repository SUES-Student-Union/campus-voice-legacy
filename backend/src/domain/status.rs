use crate::error::{AppError, AppResult};

pub const STATUS_PENDING: &str = "待受理";
pub const STATUS_SUPPLEMENT: &str = "待补充";
pub const STATUS_ACCEPTED: &str = "已受理";
pub const STATUS_PROCESSING: &str = "部门处理中";
pub const STATUS_REVIEW: &str = "待审核";
pub const STATUS_COMPLETED: &str = "已办结";
pub const STATUS_CLOSED: &str = "已关闭";

pub const ALL_STATUSES: [&str; 7] = [
    STATUS_PENDING,
    STATUS_SUPPLEMENT,
    STATUS_ACCEPTED,
    STATUS_PROCESSING,
    STATUS_REVIEW,
    STATUS_COMPLETED,
    STATUS_CLOSED,
];

pub fn allowed_transitions(from: &str) -> &'static [&'static str] {
    match from {
        STATUS_PENDING => &[STATUS_ACCEPTED, STATUS_SUPPLEMENT, STATUS_CLOSED],
        STATUS_SUPPLEMENT => &[STATUS_ACCEPTED, STATUS_CLOSED],
        STATUS_ACCEPTED => &[STATUS_PROCESSING, STATUS_SUPPLEMENT, STATUS_CLOSED],
        STATUS_PROCESSING => &[STATUS_REVIEW, STATUS_SUPPLEMENT, STATUS_CLOSED],
        STATUS_REVIEW => &[STATUS_COMPLETED, STATUS_PROCESSING, STATUS_CLOSED],
        _ => &[],
    }
}

/// 校验状态流转；`待补充 -> 待补充`（再次催补）视为合法。
pub fn assert_transition(from: &str, to: &str) -> AppResult<()> {
    if from == to && from == STATUS_SUPPLEMENT {
        return Ok(());
    }
    if from == to {
        return Err(AppError::Conflict("诉求状态未发生变化".into()));
    }
    if allowed_transitions(from).contains(&to) {
        Ok(())
    } else {
        Err(AppError::Conflict(format!(
            "状态不允许从“{from}”流转到“{to}”"
        )))
    }
}
