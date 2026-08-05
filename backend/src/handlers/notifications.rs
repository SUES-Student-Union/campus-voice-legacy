use crate::error::AppResult;
use crate::guards::AuthUser;
use crate::models::{Notification, NotificationOut};
use crate::state::AppState;
use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;

pub async fn list_notifications(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<Vec<NotificationOut>>> {
    let rows = sqlx::query_as::<_, Notification>(
        "SELECT * FROM notifications WHERE user_id = $1 ORDER BY created_at DESC LIMIT 50",
    )
    .bind(user.id())
    .fetch_all(&state.db)
    .await?;
    Ok(Json(
        rows.into_iter()
            .map(|n| NotificationOut {
                id: n.id,
                title: n.title,
                content: n.content,
                kind: n.kind,
                read: n.read,
                issue_id: n.issue_id,
                created_at: n.created_at,
            })
            .collect(),
    ))
}

pub async fn mark_notification_read(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<NotificationOut>> {
    let row = sqlx::query_as::<_, Notification>(
        "UPDATE notifications SET read = true WHERE id = $1 AND user_id = $2 RETURNING *",
    )
    .bind(id)
    .bind(user.id())
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| crate::error::AppError::NotFound("通知不存在".into()))?;
    Ok(Json(NotificationOut {
        id: row.id,
        title: row.title,
        content: row.content,
        kind: row.kind,
        read: row.read,
        issue_id: row.issue_id,
        created_at: row.created_at,
    }))
}
