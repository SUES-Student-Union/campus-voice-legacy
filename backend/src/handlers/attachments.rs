use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::handlers::get_issue;
use crate::models::{Attachment, AttachmentOut};
use crate::state::AppState;
use axum::body::Body;
use axum::extract::{Multipart, Path, State};
use axum::http::{header, HeaderValue, Response, StatusCode};
use axum::Json;
use uuid::Uuid;

pub async fn upload_attachment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(issue_id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<Vec<AttachmentOut>>> {
    let issue = get_issue(&state.db, issue_id).await?;
    user.check(
        crate::permissions::ATTACHMENT_UPLOAD,
        &crate::permissions::Ctx::for_issue(&issue),
    )?;

    let existing: i64 = sqlx::query_scalar("SELECT count(*) FROM attachments WHERE issue_id = $1")
        .bind(issue_id)
        .fetch_one(&state.db)
        .await?;
    let remain = state.cfg.max_attachments as i64 - existing;
    if remain <= 0 {
        return Err(AppError::PayloadTooLarge(format!(
            "每诉求最多 {} 张图片",
            state.cfg.max_attachments
        )));
    }

    let upload_dir = state.cfg.upload_dir.clone();
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(AppError::internal)?;

    let mut saved: Vec<AttachmentOut> = Vec::new();
    while let Some(field) = multipart.next_field().await.map_err(AppError::internal)? {
        if saved.len() as i64 >= remain {
            break;
        }
        let Some(original_name) = field.file_name().map(|s| s.to_string()) else {
            continue;
        };
        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".into());
        if !content_type.starts_with("image/") {
            return Err(AppError::BadRequest("仅支持图片附件".into()));
        }
        let data = field.bytes().await.map_err(AppError::internal)?;
        if data.len() as u64 > state.cfg.max_upload_bytes {
            return Err(AppError::PayloadTooLarge(format!(
                "图片超过 {}MB 上限",
                state.cfg.max_upload_bytes / 1024 / 1024
            )));
        }

        let ext = std::path::Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("img");
        let stored_name = format!("{}.{}", Uuid::new_v4().simple(), sanitize_ext(ext));
        tokio::fs::write(upload_dir.join(&stored_name), &data)
            .await
            .map_err(AppError::internal)?;

        let att = sqlx::query_as::<_, Attachment>(
            "INSERT INTO attachments (issue_id, uploader_id, original_name, stored_name, size, mime)
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
        )
        .bind(issue_id)
        .bind(user.id())
        .bind(&original_name)
        .bind(&stored_name)
        .bind(data.len() as i64)
        .bind(&content_type)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::internal)?;

        saved.push(AttachmentOut {
            id: att.id,
            original_name: att.original_name,
            size: att.size,
            mime: att.mime,
            created_at: att.created_at,
        });
    }

    if saved.is_empty() {
        return Err(AppError::BadRequest("未接收到有效图片".into()));
    }
    Ok(Json(saved))
}

pub async fn download_attachment(
    State(state): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Response<Body>> {
    let att = sqlx::query_as::<_, Attachment>("SELECT * FROM attachments WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("附件不存在".into()))?;

    let allowed = if att.is_public {
        true
    } else {
        match att.issue_id {
            Some(iid) => {
                let issue = get_issue(&state.db, iid).await?;
                user.check(
                    crate::permissions::ATTACHMENT_DOWNLOAD,
                    &crate::permissions::Ctx::for_issue(&issue),
                )
                .is_ok()
            }
            None => false,
        }
    };
    if !allowed {
        return Err(AppError::Forbidden("无权访问该附件".into()));
    }

    let path = state.cfg.upload_dir.join(&att.stored_name);
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|_| AppError::NotFound("附件文件已丢失".into()))?;

    let mut resp = Response::new(Body::from(bytes));
    *resp.status_mut() = StatusCode::OK;
    if let Ok(ct) = HeaderValue::from_str(&att.mime) {
        resp.headers_mut().insert(header::CONTENT_TYPE, ct);
    }
    if let Ok(cd) = HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        sanitize_filename(&att.original_name)
    )) {
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, cd);
    }
    resp.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=3600"),
    );
    Ok(resp)
}

fn sanitize_ext(ext: &str) -> String {
    ext.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
