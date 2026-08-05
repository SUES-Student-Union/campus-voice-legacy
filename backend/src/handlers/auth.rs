use crate::auth;
use crate::error::{AppError, AppResult};
use crate::guards::AuthUser;
use crate::models::{AuthResponse, LoginReq, SsoLoginReq, User, UserOut, WechatLoginReq};
use crate::state::AppState;
use axum::extract::State;
use axum::Json;
use uuid::Uuid;

pub async fn wechat_login(
    State(state): State<AppState>,
    Json(req): Json<WechatLoginReq>,
) -> AppResult<Json<AuthResponse>> {
    if req.code.is_empty() {
        return Err(AppError::BadRequest("缺少微信登录 code".into()));
    }
    let openid = auth::wechat_openid(&state.cfg, &state.http, &req.code).await?;
    let user = upsert_student(&state, &openid).await?;
    let token = auth::sign_token(
        &state.cfg,
        user.id,
        &user.role,
        &user.name,
        user.department_id,
    )?;
    Ok(Json(AuthResponse {
        token,
        user: UserOut::from(&user),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginReq>,
) -> AppResult<Json<AuthResponse>> {
    if req.username.is_empty() || req.password.is_empty() {
        return Err(AppError::BadRequest("用户名和密码不能为空".into()));
    }
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&req.username)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".into()))?;
    let hash = user
        .password_hash
        .as_deref()
        .ok_or_else(|| AppError::Unauthorized("该账号不支持密码登录".into()))?;
    if !auth::verify_password(&req.password, hash) {
        return Err(AppError::Unauthorized("用户名或密码错误".into()));
    }
    let token = auth::sign_token(
        &state.cfg,
        user.id,
        &user.role,
        &user.name,
        user.department_id,
    )?;
    Ok(Json(AuthResponse {
        token,
        user: UserOut::from(&user),
    }))
}

pub async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<UserOut>> {
    let u = crate::handlers::get_user(&state.db, user.id()).await?;
    Ok(Json(UserOut::from(&u)))
}

/// 校园统一认证登录：学号/工号+密码 -> 学校认证 -> 绑定微信 openid -> JWT。
pub async fn sso_login(
    State(state): State<AppState>,
    Json(req): Json<SsoLoginReq>,
) -> AppResult<Json<AuthResponse>> {
    let openid = auth::wechat_openid(&state.cfg, &state.http, &req.code).await?;
    let profile = crate::sso::verify(&req.campus_id, &req.password).await?;

    let dept_id = match profile.department_code.as_deref() {
        Some(code) => {
            sqlx::query_scalar::<_, Uuid>("SELECT id FROM departments WHERE code = $1")
                .bind(code)
                .fetch_optional(&state.db)
                .await?
        }
        None => None,
    };

    // 绑定 openid：先清理其他账号对同一 openid 的占用（开发环境为单一 openid）。
    sqlx::query("UPDATE users SET openid = NULL WHERE openid = $1")
        .bind(&openid)
        .execute(&state.db)
        .await?;

    let user = if let Some(existing) =
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE campus_id = $1")
            .bind(&profile.campus_id)
            .fetch_optional(&state.db)
            .await?
    {
        sqlx::query_as::<_, User>(
            "UPDATE users SET openid = $1, role = $2, name = $3, department_id = $4, updated_at = now()
             WHERE id = $5 RETURNING *",
        )
        .bind(&openid)
        .bind(&profile.role)
        .bind(&profile.name)
        .bind(dept_id)
        .bind(existing.id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::internal)?
    } else {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (campus_id, openid, role, name, department_id)
             VALUES ($1, $2, $3, $4, $5) RETURNING *",
        )
        .bind(&profile.campus_id)
        .bind(&openid)
        .bind(&profile.role)
        .bind(&profile.name)
        .bind(dept_id)
        .fetch_one(&state.db)
        .await
        .map_err(AppError::internal)?
    };

    let token = auth::sign_token(
        &state.cfg,
        user.id,
        &user.role,
        &user.name,
        user.department_id,
    )?;
    Ok(Json(AuthResponse {
        token,
        user: UserOut::from(&user),
    }))
}

async fn upsert_student(state: &AppState, openid: &str) -> AppResult<User> {
    if let Some(u) = sqlx::query_as::<_, User>("SELECT * FROM users WHERE openid = $1")
        .bind(openid)
        .fetch_optional(&state.db)
        .await?
    {
        return Ok(u);
    }
    let suffix: String = openid
        .chars()
        .rev()
        .take(4)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let name = format!("同学{suffix}");
    sqlx::query_as::<_, User>(
        "INSERT INTO users (openid, role, name) VALUES ($1, 'student', $2) RETURNING *",
    )
    .bind(openid)
    .bind(&name)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("创建学生账号失败: {e}")))
}
